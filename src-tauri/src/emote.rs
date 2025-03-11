use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use axum::http::StatusCode;
use lazy_static::lazy_static;
use log::{error, info};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;

use crate::api::{send_gql, HTTP_CLIENT};

const TURBO_AND_SUB_UPSELL_QUERY_HASH: &str =
    "5dbca380e47e37808c89479f51f789990ec653428a01b76c649ebe01afb3aa7e";

const SEVENTV_API: &str = "https://7tv.io/v3";
const BETTERTV_API: &str = "https://api.betterttv.net/3";

#[derive(Serialize, Default, Clone, Debug)]
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
    pub static ref EMOTES_CACHE: Mutex<HashMap<String, HashMap<String, Emote>>> =
        Mutex::new(HashMap::new());
}

pub async fn get_user_emotes(username: &str, id: &str) -> Result<()> {
    if username.is_empty() {
        return Err(anyhow!("No username provided"));
    }

    if id.is_empty() {
        return Err(anyhow!("ID for '{username}' is empty"));
    }

    let mut cache = EMOTES_CACHE.lock().await;
    if cache.contains_key(username) {
        return Ok(());
    }

    let user_emotes = match fetch_user_emotes(username).await {
        Ok(emotes) => emotes,
        Err(err) => {
            error!("Failed to fetch user emotes: {err}");
            Vec::new()
        }
    };

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

    let mut emotes =
        HashMap::with_capacity(user_emotes.len() + seventv_emotes.len() + bettertv_emotes.len());

    emotes.extend(
        user_emotes
            .into_iter()
            .map(|emote| (emote.name.clone(), emote)),
    );

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

    info!("Updating emotes for '{username}'");
    cache
        .entry(username.to_string())
        .or_insert_with(HashMap::new)
        .extend(
            emotes
                .iter()
                .map(|(name, emote)| (name.clone(), emote.clone())),
        );

    Ok(())
}

async fn fetch_user_emotes(username: &str) -> Result<Vec<Emote>> {
    let query = json!({
        "operationName": "TurboAndSubUpsell",
        "variables": {
            "channelLogin": username
        },
        "extensions": {
            "persistedQuery": {
            "version": 1,
            "sha256Hash": TURBO_AND_SUB_UPSELL_QUERY_HASH
            }
        }
    });

    let response = match send_gql(query).await {
        Ok(response) => response,
        Err(err) => return Err(anyhow!("Failed to get user emotes: {err}")),
    };

    let data = match response.pointer("/data/user/subscriptionProducts") {
        Some(val) => val.as_array(),
        None => return Err(anyhow!("User subscription products not found")),
    };

    if data.is_none() {
        return Err(anyhow!("User subscription products not found"));
    }

    let data = data.unwrap();

    let mut user_emotes = Vec::new();

    for product in data {
        let product = product.as_object().unwrap();

        let emotes = match product.get("emotes") {
            Some(val) => val.as_array(),
            None => continue,
        };

        let emotes = emotes.unwrap();
        if emotes.is_empty() {
            continue;
        }

        for emote in emotes {
            // Not sure if another type can exist here, so checking if __typename is equal to Emote.
            if emote.get("__typename").unwrap().as_str().unwrap() != "Emote" {
                continue;
            }

            let id = emote.get("id").unwrap().as_str().unwrap();
            let token = emote.get("token").unwrap().as_str().unwrap();
            let emote = format!("https://static-cdn.jtvnw.net/emoticons/v2/{id}/default/dark/1.0");

            user_emotes.push(Emote {
                name: token.to_string(),
                url: emote,
                width: 28,
                height: 28,
            });
        }
    }

    Ok(user_emotes)
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
    let response = HTTP_CLIENT
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
