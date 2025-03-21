use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use axum::http::StatusCode;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::Row;

use crate::{APP_STATE, HTTP_CLIENT};

pub const TWITCH_EMOTES_CDN: &str = "https://static-cdn.jtvnw.net/emoticons/v2";
const SEVENTV_API: &str = "https://7tv.io/v3";
const BETTERTV_API: &str = "https://api.betterttv.net/3";

pub async fn query_emotes(username: &str) -> Result<HashMap<String, Emote>, String> {
    let state = APP_STATE.lock().await;
    let db = state.emotes_db.as_ref().ok_or("Database not initialized")?;

    let query = "SELECT name, url, width, height FROM emotes WHERE username = ?";

    let rows = sqlx::query(query)
        .bind(username)
        .fetch_all(db)
        .await
        .map_err(|e| e.to_string())?;

    let mut emotes = HashMap::new();

    for row in rows {
        let emote_name: String = row.try_get("name").map_err(|e| e.to_string())?;
        let url: String = row.try_get("url").map_err(|e| e.to_string())?;
        let width: i64 = row.try_get("width").map_err(|e| e.to_string())?;
        let height: i64 = row.try_get("height").map_err(|e| e.to_string())?;

        let emote = Emote {
            name: emote_name.clone(),
            url,
            width,
            height,
        };

        emotes.insert(emote_name, emote);
    }

    Ok(emotes)
}

pub async fn save_emotes(username: &str, emotes: HashMap<String, Emote>) -> Result<()> {
    let state = APP_STATE.lock().await;
    let db = state.emotes_db.as_ref().expect("Database not initialized");

    let mut tx = db.begin().await?;

    sqlx::query("DELETE FROM emotes WHERE username = ?")
        .bind(username)
        .execute(&mut *tx)
        .await?;

    if emotes.is_empty() {
        tx.commit().await?;
        return Ok(());
    }

    let emote_values: Vec<&Emote> = emotes.values().collect();

    let mut query_str =
        String::from("INSERT INTO emotes (username, name, url, width, height) VALUES ");

    let placeholders: Vec<String> = emote_values
        .iter()
        .map(|_| "(?, ?, ?, ?, ?)".to_string())
        .collect();

    query_str.push_str(&placeholders.join(", "));

    let mut sql_query = sqlx::query(&query_str);
    for emote in emote_values {
        sql_query = sql_query
            .bind(username)
            .bind(&emote.name)
            .bind(&emote.url)
            .bind(emote.width)
            .bind(emote.height);
    }

    sql_query.execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
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

pub async fn fetch_bettertv_emotes(id: &str) -> Result<HashMap<String, Emote>> {
    let response = fetch_and_deserialize::<BetterTTVResponse>(&format!(
        "{BETTERTV_API}/cached/users/twitch/{id}"
    ))
    .await?;

    let raw_emotes = [&response.channel_emotes[..], &response.shared_emotes[..]].concat();

    let mut emotes: HashMap<String, Emote> = HashMap::new();

    for emote in raw_emotes {
        let name = emote.code;
        let url = format!("https://cdn.betterttv.net/emote/{}/1x", emote.id);

        let emote = Emote {
            name: name.clone(),
            url,
            width: emote.width.unwrap_or(28),
            height: emote.height.unwrap_or(28),
        };

        emotes.insert(name, emote);
    }

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

pub async fn fetch_7tv_emotes(id: &str) -> Result<HashMap<String, Emote>> {
    let response =
        fetch_and_deserialize::<SevenTVResponse>(&format!("{SEVENTV_API}/users/twitch/{id}"))
            .await?;

    let mut emotes: HashMap<String, Emote> = HashMap::new();

    for mut emote in response.emote_set.emotes {
        emote
            .data
            .host
            .files
            .retain(|file| file.name.starts_with('1'));

        if emote.data.host.files.is_empty() {
            continue;
        }

        let host = emote.data.host;
        let name = emote.name.clone();

        // Define a closure to assign a priority to each file format
        let priority = |format: &str| match format.to_uppercase().as_str() {
            "AVIF" => Some(0),
            "WEBP" => Some(1),
            "PNG" => Some(2),
            "GIF" => Some(3),
            _ => None,
        };

        // Find the file with the highest priority (lowest number)
        let mut best_priority: Option<usize> = None;
        let mut best_file: Option<&_> = None;
        for file in &host.files {
            if let Some(p) = priority(&file.format) {
                if best_priority.is_none() || p < best_priority.unwrap() {
                    best_priority = Some(p);
                    best_file = Some(file);
                }
            }
        }

        if let Some(file) = best_file {
            let new_emote = Emote {
                name: name.clone(),
                url: format!("https:{}/{}", host.url, file.name),
                width: file.width,
                height: file.height,
            };
            emotes.insert(name, new_emote);
        }
    }

    Ok(emotes)
}

async fn fetch_and_deserialize<T: DeserializeOwned>(url: &str) -> Result<T> {
    let response = HTTP_CLIENT
        .get(url)
        .send()
        .await
        .context("Failed to send emotes request")?;

    let status = response.status();

    if !status.is_success() && status == StatusCode::NOT_FOUND {
        let error_body = response
            .text()
            .await
            .context("Failed to read response body")?;

        return Err(anyhow!("Request failed with status {status}: {error_body}"));
    }

    response
        .json()
        .await
        .context("Failed to deserialize emotes response")
}
