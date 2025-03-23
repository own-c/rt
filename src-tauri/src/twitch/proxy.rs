use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use tauri::{async_runtime::Mutex, AppHandle, Emitter};

use super::{main::PROXY_HTTP_CLIENT, stream};

lazy_static! {
    // These are public so that they can be reset when changing streams in the tauri commands.
    pub static ref USING_BACKUP: AtomicBool = AtomicBool::new(false);
    pub static ref MAIN_STREAM_URL: Mutex<Option<String>> = Mutex::new(None);
    pub static ref BACKUP_STREAM_URL: Mutex<Option<String>> = Mutex::new(None);

    static ref URL_REGEX: Regex = Regex::new(r"^(https?://[^\s]+)").unwrap();
}

#[tauri::command]
pub async fn proxy_stream(
    app_handle: AppHandle,
    username: &str,
    url: &str,
) -> Result<String, String> {
    if url.is_empty() {
        return Err(String::from("No URL provided"));
    }

    let response = match PROXY_HTTP_CLIENT.get(url).send().await {
        Ok(resp) => resp,
        Err(err) => {
            return Err(format!("Failed to proxy request: {err}"));
        }
    };

    let body_bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err(format!("Failed to read response body: {err}"));
        }
    };

    let (ad_detected, mut playlist, is_master_playlist) =
        process_m3u8(&String::from_utf8_lossy(&body_bytes));

    let using_backup = USING_BACKUP.load(Ordering::Relaxed);

    if !is_master_playlist {
        {
            let mut main_stream = MAIN_STREAM_URL.lock().await;
            if main_stream.is_none() {
                *main_stream = Some(url.to_string());
            }
        }

        if ad_detected {
            // On ad detection, if backup isn’t already enabled, switch to backup
            if !using_backup {
                info!("Found ad in variant playlist. Switching to backup stream.");

                if let Err(err) = app_handle.emit("stream", "backup") {
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
                    let url = fetch_backup_stream_url(username).await.unwrap_or_default();
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

            if let Err(err) = app_handle.emit("stream", "main") {
                error!("Failed to emit event: {err}");
            }

            match fetch_main_stream(username).await {
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

    Ok(playlist)
}

fn process_m3u8(playlist: &str) -> (bool, String, bool) {
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
    let (ad_detected, processed_playlist, _) = process_m3u8(&body);

    if ad_detected {
        USING_BACKUP.store(true, Ordering::Relaxed);
        return Ok("#EXTM3U\n#EXT-X-ENDLIST\n".to_string());
    }

    USING_BACKUP.store(false, Ordering::Relaxed);
    Ok(processed_playlist)
}

async fn fetch_backup_stream_url(username: &str) -> Result<String> {
    let url = match stream::fetch_stream_playback(username, true).await {
        Ok(url) => url,
        Err(err) => {
            return Err(anyhow!("Failed to fetch backup stream: {err}"));
        }
    };

    let body = fetch_playlist_text(&url).await?;

    let variant_playlist_url = body
        .lines()
        .nth(4)
        .ok_or_else(|| anyhow!("Backup master playlist is malformed."))?
        .to_string();

    Ok(variant_playlist_url)
}
