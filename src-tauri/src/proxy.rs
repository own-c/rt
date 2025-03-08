use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{anyhow, Result};
use axum::{
    body::{Body, HttpBody},
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use serde::Deserialize;
use tauri::Url;
use tauri_plugin_http::reqwest::{header::HeaderValue, Client};
use tokio::sync::Mutex;

use crate::{api::LOCAL_API, user};

lazy_static! {
    pub static ref PROXY_HTTP_CLIENT: Client = Client::builder()
        .gzip(true)
        .use_rustls_tls()
        .https_only(true)
        .build()
        .unwrap();
    pub static ref USING_BACKUP: AtomicBool = AtomicBool::new(false);
    static ref MAIN_STREAM_URL: Mutex<Option<String>> = Mutex::new(None);
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

    let mut headers = response.headers().clone();

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    let body = if content_type.contains("application/vnd.apple.mpegurl") {
        let body_bytes = response.bytes().await.unwrap_or(Bytes::new());

        let (ad_detected, mut playlist, is_master_playlist, stream_url) =
            process_m3u8(&username, &url, &String::from_utf8_lossy(&body_bytes));

        if !is_master_playlist {
            let mut main_stream = MAIN_STREAM_URL.lock().await;

            if main_stream.is_none() {
                *main_stream = Some(stream_url.clone());
            }

            if ad_detected {
                if !USING_BACKUP.load(Ordering::SeqCst) {
                    info!("Found AD in variant playlist. Switching to backup stream.");
                    USING_BACKUP.store(true, Ordering::SeqCst);
                }

                match fetch_backup_stream(&username).await {
                    Ok(pl) => playlist = pl,
                    Err(err) => {
                        error!("Failed to fetch backup stream: {err}");
                        playlist.clear();
                    }
                }
            } else if USING_BACKUP.load(Ordering::SeqCst) {
                info!("No AD detected. Switching back to main stream.");
                USING_BACKUP.store(false, Ordering::SeqCst);

                match fetch_main_stream(&username).await {
                    Ok(pl) => playlist = pl,
                    Err(err) => {
                        error!("Failed to fetch main stream: {err}");
                        playlist.clear();
                    }
                }
            }
        }
        Body::from(playlist)
    } else {
        Body::from(response.bytes().await.unwrap_or(Bytes::new()))
    };

    let mut resp = Response::new(Body::default());

    if content_type.contains("stream") {
        *resp.body_mut() = Body::from_stream(body.into_data_stream());
    } else {
        let new_content_length = match body.size_hint().exact() {
            Some(size) => size.to_string(),
            None => "0".to_string(),
        };

        headers.insert(
            "content-length",
            HeaderValue::from_str(&new_content_length).unwrap(),
        );

        *resp.body_mut() = body;
    }

    *resp.headers_mut() = headers.clone();

    (StatusCode::OK, resp)
}

fn process_m3u8(username: &str, base_url: &str, playlist: &str) -> (bool, String, bool, String) {
    let base_url = Url::parse(base_url).ok();
    let mut result_lines = Vec::new();
    let mut ad_detected = false;
    let mut is_master_playlist = false;
    let mut stream_url = String::new();

    for line in playlist.lines() {
        if line.starts_with("#EXT-X-STREAM-INF") {
            is_master_playlist = true;
        }

        if line.contains("stitched-ad") {
            ad_detected = true;
        }

        if !is_master_playlist && !line.starts_with('#') && !line.trim().is_empty() {
            stream_url = line.to_string();
        }

        if URL_REGEX.is_match(line) || (!line.starts_with('#') && !line.is_empty()) {
            if let Some(base) = &base_url {
                if let Ok(abs_url) = base.join(line) {
                    result_lines.push(format!(
                        "{LOCAL_API}/proxy?username={username}&url={}",
                        urlencoding::encode(abs_url.as_str())
                    ));

                    continue;
                }
            }
        }

        result_lines.push(line.to_string());
    }

    (
        ad_detected,
        result_lines.join("\n"),
        is_master_playlist,
        stream_url,
    )
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
        return fetch_backup_stream(username).await;
    };

    let body = fetch_playlist_text(&main_url).await?;
    let (ad_detected, processed_playlist, _, _) = process_m3u8(username, &main_url, &body);

    if ad_detected {
        info!("Ads still present in main stream. Using backup stream.");
        fetch_backup_stream(username).await
    } else {
        info!("Ads cleared in main stream. Switching back.");
        USING_BACKUP.store(false, Ordering::SeqCst);
        Ok(processed_playlist)
    }
}

async fn fetch_backup_stream(username: &str) -> Result<String> {
    let stream = user::fetch_stream(username, true).await?;

    let backup_url = stream.url.replace(
        format!("{LOCAL_API}/proxy?username={username}&url=").as_str(),
        "",
    );

    let decoded_url = urlencoding::decode(&backup_url)?.to_string();

    let body = fetch_playlist_text(&decoded_url).await?;
    let variant_playlist_url = body
        .lines()
        .nth(4)
        .ok_or_else(|| anyhow!("Backup master playlist is malformed."))?
        .to_string();

    fetch_playlist_text(&variant_playlist_url).await
}
