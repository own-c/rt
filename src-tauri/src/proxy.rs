use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use serde::Deserialize;
use tauri::{Emitter, Url};
use tokio::sync::Mutex;

use crate::{user, APP_HANDLE, LOCAL_API_ADDR, PROXY_HTTP_CLIENT};

lazy_static! {
    // These are public so that they can be reset when changing streams in the tauri commands.
    pub static ref USING_BACKUP: AtomicBool = AtomicBool::new(false);
    pub static ref MAIN_STREAM_URL: Mutex<Option<String>> = Mutex::new(None);
    pub static ref BACKUP_STREAM_URL: Mutex<Option<String>> = Mutex::new(None);

    static ref URL_REGEX: Regex = Regex::new(r"^(https?://[^\s]+)").unwrap();
}

#[derive(Deserialize)]
pub struct ProxyStreamQuery {
    url: String,
    username: String,
}

pub async fn proxy_stream(Query(query): Query<ProxyStreamQuery>) -> impl IntoResponse {
    let url = query.url;
    let username = query.username;

    if url.is_empty() {
        error!("No URL provided");
        return (StatusCode::BAD_REQUEST, Response::default());
    }

    let response = match PROXY_HTTP_CLIENT.get(&url).send().await {
        Ok(resp) => resp,
        Err(err) => {
            error!("Proxying request: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Response::new(Body::default()),
            );
        }
    };

    let body_bytes = response.bytes().await.unwrap_or(Bytes::new());

    let (ad_detected, mut playlist, is_master_playlist) =
        process_m3u8(&username, &url, &String::from_utf8_lossy(&body_bytes));

    let using_backup = USING_BACKUP.load(Ordering::Relaxed);

    if !is_master_playlist {
        {
            let mut main_stream = MAIN_STREAM_URL.lock().await;
            if main_stream.is_none() {
                *main_stream = Some(url);
            }
        }

        if ad_detected {
            // On ad detection, if backup isn’t already enabled, switch to backup
            if !using_backup {
                info!("Found ad in variant playlist. Switching to backup stream.");

                if let Err(err) = APP_HANDLE
                    .lock()
                    .await
                    .as_ref()
                    .unwrap()
                    .emit("stream", "backup")
                {
                    error!("Failed to emit event: {err}");
                }

                USING_BACKUP.store(true, Ordering::Relaxed);
            }

            // Use the cached backup stream URL if available, if not, fetch it once
            let backup_url = {
                let mut backup_url_guard = BACKUP_STREAM_URL.lock().await;

                if let Some(url) = backup_url_guard.clone() {
                    url
                } else {
                    let url = fetch_backup_stream_url(&username).await.unwrap_or_default();
                    *backup_url_guard = Some(url.clone());
                    url
                }
            };

            // Fetch an updated backup manifest from the cached backup URL
            match fetch_playlist_text(&backup_url).await {
                Ok(updated_playlist) => playlist = updated_playlist,
                Err(err) => {
                    error!("Failed to fetch updated backup manifest: {err}");
                    playlist.clear();
                }
            }
        } else if using_backup {
            // If no ad is detected but we are still in backup, switch back to the main stream
            info!("No ad detected. Switching back to main stream.");

            if let Err(err) = APP_HANDLE
                .lock()
                .await
                .as_ref()
                .unwrap()
                .emit("stream", "main")
            {
                error!("Failed to emit event: {err}");
            }

            match fetch_main_stream(&username).await {
                Ok(pl) => playlist = pl,
                Err(err) => {
                    error!("Failed to fetch main stream: {err}");
                    playlist.clear();
                }
            }

            USING_BACKUP.store(false, Ordering::Relaxed);
            *BACKUP_STREAM_URL.lock().await = None;
        }
    }

    let new_response = Response::new(Body::from(playlist));

    (StatusCode::OK, new_response)
}

fn process_m3u8(username: &str, base_url: &str, playlist: &str) -> (bool, String, bool) {
    let base_url = Url::parse(base_url).ok();
    let mut result_lines = Vec::new();
    let mut ad_detected = false;
    let mut is_master_playlist = false;

    for line in playlist.lines() {
        if line.starts_with("#EXT-X-STREAM-INF") {
            is_master_playlist = true;
        }

        if line.contains("stitched-ad") {
            ad_detected = true;
        }

        // Check if line looks like a URL or a non-comment non-empty line
        if URL_REGEX.is_match(line) || (!line.starts_with('#') && !line.is_empty()) {
            if let Some(base) = &base_url {
                if let Ok(abs_url) = base.join(line) {
                    // Only add the proxy prefix if the URL contains .m3u8
                    if abs_url.as_str().to_lowercase().contains(".m3u8") {
                        result_lines.push(format!(
                            "http://{LOCAL_API_ADDR}/proxy?username={username}&url={}",
                            urlencoding::encode(abs_url.as_str())
                        ));
                    } else {
                        result_lines.push(abs_url.to_string());
                    }

                    continue;
                }
            }
        }

        result_lines.push(line.to_string());
    }

    (ad_detected, result_lines.join("\n"), is_master_playlist)
}

async fn fetch_playlist_text(url: &str) -> Result<String> {
    let response = PROXY_HTTP_CLIENT
        .get(url)
        .send()
        .await
        .map_err(|err| anyhow!("Failed to fetch: {err}"))?;

    response
        .text()
        .await
        .map_err(|err| anyhow!("Failed to read text: {err}"))
}

async fn fetch_main_stream(username: &str) -> Result<String> {
    let Some(main_url) = MAIN_STREAM_URL.lock().await.clone() else {
        error!("Main stream URL not found. Falling back to backup stream.");
        let backup_url = fetch_backup_stream_url(username).await?;
        return fetch_playlist_text(&backup_url).await;
    };

    let body = fetch_playlist_text(&main_url).await?;
    let (ad_detected, processed_playlist, _) = process_m3u8(username, &main_url, &body);

    if ad_detected {
        USING_BACKUP.store(true, Ordering::Relaxed);
        return Ok("#EXTM3U\n#EXT-X-ENDLIST\n".to_string());
    }

    USING_BACKUP.store(false, Ordering::Relaxed);
    Ok(processed_playlist)
}

async fn fetch_backup_stream_url(username: &str) -> Result<String> {
    let url = match user::fetch_stream_playback(username, true).await {
        Ok(url) => url,
        Err(err) => {
            return Err(anyhow!("Failed to fetch backup stream: {err}"));
        }
    };

    let backup_url = url.replace(
        format!("http://{LOCAL_API_ADDR}/proxy?username={username}&url=").as_str(),
        "",
    );

    let decoded_url = urlencoding::decode(&backup_url)?.to_string();

    let body = fetch_playlist_text(&decoded_url).await?;

    let variant_playlist_url = body
        .lines()
        .nth(4)
        .ok_or_else(|| anyhow!("Backup master playlist is malformed."))?
        .to_string();

    Ok(variant_playlist_url)
}
