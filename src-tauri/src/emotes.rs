use std::collections::HashMap;

use bytes::Bytes;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tauri_plugin_http::reqwest;
use tokio::sync::Mutex;

use crate::api::Emote;

const SEVENTV_API: &str = "https://7tv.io/v3";
const BETTERTV_API: &str = "https://api.betterttv.net/3";

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
    static ref CACHE: Mutex<HashMap<String, Vec<Emote>>> = Mutex::new(HashMap::new());
}

pub async fn fetch_emotes(id: &str) -> Vec<Emote> {
    let mut lock = CACHE.lock().await;
    if let Some(emotes) = lock.get(id) {
        println!("Emotes cache hit for {}", id);
        return emotes.clone();
    }

    let mut emotes = Vec::new();

    let seventv_emotes = fetch_7tv_emotes(id).await;
    let bettertv_emotes = fetch_better_tv_emotes(id).await;

    emotes.extend(seventv_emotes);
    emotes.extend(bettertv_emotes);

    lock.insert(id.to_string(), emotes.clone());

    emotes
}

#[derive(Deserialize, Default)]
pub struct BetterTTV {
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

async fn fetch_better_tv_emotes(id: &str) -> Vec<Emote> {
    let response = HTTP_CLIENT
        .get(format!("{BETTERTV_API}/cached/users/twitch/{id}"))
        .send()
        .await;

    if let Err(err) = response {
        println!("Error bettertv fetching emotes: {err}");
        return Vec::<Emote>::default();
    }

    let response = response.unwrap();
    if response.status() != 200 {
        println!(
            "Error bettertv fetching emotes: status code {}",
            response.status()
        );
        return Vec::<Emote>::default();
    }

    let body = response.bytes().await.unwrap_or(Bytes::new());
    let raw_emotes = match serde_json::from_slice::<BetterTTV>(&body) {
        Ok(data) => [&data.channel_emotes[..], &data.shared_emotes[..]].concat(),
        Err(err) => {
            println!("Error deserializing bettertv emotes: {}", err);
            return Vec::<Emote>::default();
        }
    };

    raw_emotes
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
        .collect()
}

#[derive(Serialize, Deserialize)]
struct SevenTVResponse {
    emote_set: SevenTVEmoteSet,
}

#[derive(Serialize, Deserialize)]
struct SevenTVEmoteSet {
    emotes: Vec<SevenTVEmote>,
}

#[derive(Serialize, Deserialize)]
struct SevenTVEmote {
    name: String,
    data: SevenTVEmoteData,
}

#[derive(Serialize, Deserialize)]
struct SevenTVEmoteData {
    animated: bool,
    host: SevenTVEmoteDataHost,
}

#[derive(Serialize, Deserialize)]
struct SevenTVEmoteDataHost {
    url: String,
    files: Vec<SevenTVEmoteDataHostFile>,
}

#[derive(Serialize, Deserialize)]
struct SevenTVEmoteDataHostFile {
    name: String,
    width: i64,
    height: i64,
    format: String,
}

async fn fetch_7tv_emotes(id: &str) -> Vec<Emote> {
    let response = HTTP_CLIENT
        .get(format!("{SEVENTV_API}/users/twitch/{id}"))
        .send()
        .await;

    if let Err(err) = response {
        println!("Error 7tv fetching emotes: {err}");
        return Vec::<Emote>::default();
    }

    let response = response.unwrap();
    if response.status() != 200 {
        println!(
            "Error 7tv fetching emotes: status code {}",
            response.status()
        );
        return Vec::<Emote>::default();
    }

    let body = response.bytes().await.unwrap_or(Bytes::new());
    let raw_emotes = match serde_json::from_slice::<SevenTVResponse>(&body) {
        Ok(data) => data.emote_set.emotes,
        Err(err) => {
            println!("Error deserializing 7tv emotes: {}", err);
            return Vec::<Emote>::default();
        }
    };

    raw_emotes
        .into_iter()
        .filter_map(|mut emote| {
            emote
                .data
                .host
                .files
                .retain(|file| file.name.starts_with("1"));

            if !emote.data.host.files.is_empty() {
                Some(emote)
            } else {
                None
            }
        })
        .collect::<Vec<SevenTVEmote>>()
        .into_iter()
        .filter_map(|emote| {
            let name = emote.name;
            let data = emote.data;

            let host = data.host;

            fn priority(format: &str) -> Option<u8> {
                match format.to_uppercase().as_str() {
                    "AVIF" => Some(0),
                    "WEBP" => Some(1),
                    "PNG" => Some(2),
                    "GIF" => Some(3),
                    _ => None,
                }
            }

            let file = host
                .files
                .iter()
                .filter_map(|file| priority(&file.format).map(|p| (p, file)))
                .min_by_key(|(p, _)| *p)
                .map(|(_, file)| (file));

            if let Some(file) = file {
                let url = format!("https:{}/{}", host.url, file.name);
                return Some(Emote {
                    name,
                    url,
                    width: file.width,
                    height: file.height,
                });
            }

            None
        })
        .collect()
}
