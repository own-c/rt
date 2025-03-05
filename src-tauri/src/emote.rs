use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use lazy_static::lazy_static;
use log::{error, info};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tauri_plugin_http::reqwest::Client;
use tokio::sync::Mutex;

use crate::{user::USER_TO_ID, utils};

const SEVENTV_API: &str = "https://7tv.io/v3";
const BETTERTV_API: &str = "https://api.betterttv.net/3";

#[derive(Serialize, Default, Clone)]
pub struct Emote {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "u")]
    pub url: String,
    #[serde(rename = "w")]
    pub width: i64,
    #[serde(rename = "h")]
    pub height: i64,
}

lazy_static! {
    static ref EMOTE_HTTP_CLIENT: Client = utils::new_http_client();
    static ref EMOTES_CACHE: Mutex<HashMap<String, HashMap<String, Emote>>> =
        Mutex::new(HashMap::new());
}

pub async fn get_user_emotes(username: Path<String>) -> impl IntoResponse {
    let Path(username) = username;

    if username.is_empty() {
        error!("No username provided");
        return (StatusCode::BAD_REQUEST, Json(HashMap::new()));
    }

    let lock = USER_TO_ID.lock().await;

    let Some(id) = lock.get(&username) else {
        error!("No ID found for '{username}'");
        return (StatusCode::BAD_REQUEST, Json(HashMap::new()));
    };

    if id.is_empty() {
        error!("ID for '{username}' is empty");
        return (StatusCode::BAD_REQUEST, Json(HashMap::new()));
    }

    let mut lock = EMOTES_CACHE.lock().await;
    if let Some(emotes) = lock.get(id) {
        info!("Emotes cache hit for '{username}'");
        return (StatusCode::OK, Json(emotes.clone()));
    }

    let seventv_emotes = match fetch_7tv_emotes(id).await {
        Ok(emotes) => emotes,
        Err(err) => {
            error!("Failed to fetch 7tv emotes: {err}");
            Vec::new()
        }
    };

    let bettertv_emotes = match fetch_bettertv_emotes(id).await {
        Ok(emotes) => emotes,
        Err(err) => {
            error!("Failed to fetch bettertv emotes: {err}");
            Vec::new()
        }
    };

    let mut emotes = HashMap::with_capacity(seventv_emotes.len() + bettertv_emotes.len());

    emotes.extend(
        seventv_emotes
            .into_iter()
            .map(|emote| (emote.name.clone(), emote)),
    );

    emotes.extend(
        bettertv_emotes
            .into_iter()
            .map(|emote| (emote.name.clone(), emote)),
    );

    lock.insert(id.to_string(), emotes.clone());

    info!("Updated emotes for '{username}'");
    (StatusCode::OK, Json(emotes))
}

#[derive(Deserialize, Default)]
pub struct BetterTTVResponse {
    #[serde(rename = "channelEmotes")]
    channel_emotes: Vec<BetterTTVEmote>,
    #[serde(rename = "sharedEmotes")]
    shared_emotes: Vec<BetterTTVEmote>,
}

#[derive(Deserialize, Default, Clone)]
pub struct BetterTTVEmote {
    id: String,
    code: String,
    width: Option<i64>,
    height: Option<i64>,
}

async fn fetch_bettertv_emotes(id: &str) -> Result<Vec<Emote>> {
    let response = fetch_and_deserialize::<BetterTTVResponse>(&format!(
        "{BETTERTV_API}/cached/users/twitch/{id}"
    ))
    .await?;

    let raw_emotes = [&response.channel_emotes[..], &response.shared_emotes[..]].concat();

    let emotes = raw_emotes
        .into_iter()
        .map(|emote| {
            let url = format!("https://cdn.betterttv.net/emote/{}/1x", emote.id);
            Emote {
                name: emote.code,
                url,
                width: emote.width.unwrap_or(28),
                height: emote.height.unwrap_or(28),
            }
        })
        .collect();

    Ok(emotes)
}

#[derive(Deserialize)]
struct SevenTVResponse {
    emote_set: SevenTVEmoteSet,
}

#[derive(Deserialize)]
struct SevenTVEmoteSet {
    emotes: Vec<SevenTVEmote>,
}

#[derive(Deserialize)]
struct SevenTVEmote {
    name: String,
    data: SevenTVEmoteData,
}

#[derive(Deserialize)]
struct SevenTVEmoteData {
    host: SevenTVEmoteDataHost,
}

#[derive(Deserialize)]
struct SevenTVEmoteDataHost {
    url: String,
    files: Vec<SevenTVEmoteDataHostFile>,
}

#[derive(Deserialize)]
struct SevenTVEmoteDataHostFile {
    name: String,
    width: i64,
    height: i64,
    format: String,
}

async fn fetch_7tv_emotes(id: &str) -> Result<Vec<Emote>> {
    let response =
        fetch_and_deserialize::<SevenTVResponse>(&format!("{SEVENTV_API}/users/twitch/{id}"))
            .await?;

    let emotes: Vec<Emote> = response
        .emote_set
        .emotes
        .into_iter()
        .filter_map(|mut emote| {
            emote
                .data
                .host
                .files
                .retain(|file| file.name.starts_with('1'));

            (!emote.data.host.files.is_empty()).then_some(emote)
        })
        .filter_map(|emote| {
            let host = emote.data.host;
            let name = emote.name;

            let priority = |format: &str| match format.to_uppercase().as_str() {
                "AVIF" => Some(0),
                "WEBP" => Some(1),
                "PNG" => Some(2),
                "GIF" => Some(3),
                _ => None,
            };

            host.files
                .iter()
                .filter_map(|file| priority(&file.format).map(|p| (p, file)))
                .min_by_key(|(p, _)| *p)
                .map(|(_, file)| Emote {
                    name,
                    url: format!("https:{}/{}", host.url, file.name),
                    width: file.width,
                    height: file.height,
                })
        })
        .collect();

    Ok(emotes)
}

async fn fetch_and_deserialize<T: DeserializeOwned>(url: &str) -> Result<T> {
    let response = EMOTE_HTTP_CLIENT
        .get(url)
        .send()
        .await
        .context("Failed to send request")?;

    let status = response.status();

    if !status.is_success() && status == StatusCode::NOT_FOUND {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|err| format!("Unknown error: {err}"));

        return Err(anyhow!("Request failed with status {status}: {error_body}"));
    }

    let body = response
        .bytes()
        .await
        .context("Failed to read response body")?;

    if body.is_empty() {
        return Err(anyhow!("Received empty response"));
    }

    let data: T = serde_json::from_slice(&body).context("Failed to deserialize response")?;
    Ok(data)
}
