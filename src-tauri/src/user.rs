use std::{collections::HashMap, str::FromStr};

use anyhow::Result;
use log::error;
use serde::Serialize;
use tauri::{AppHandle, Url};
use tauri_plugin_store::StoreExt;

use crate::{
    emote::{self, Emote, EMOTES_CACHE, TWITCH_EMOTES_CDN},
    queries::{GraphQLQuery, GraphQLResponse, UseLiveQuery, UseLiveResponse},
    utils, LOCAL_API_ADDR,
};

const USHER_API: &str = "https://usher.ttvnw.net/api/channel/hls";
pub const BOXART_CDN: &str = "https://static-cdn.jtvnw.net/ttv-boxart";

#[derive(Serialize, Debug)]
pub struct LiveNow {
    username: String,
    started_at: String,
}

#[tauri::command]
pub async fn fetch_live_now(usernames: Vec<String>) -> Result<HashMap<String, LiveNow>, String> {
    if usernames.is_empty() {
        return Err(String::from("No usernames provided"));
    }

    let mut query: Vec<UseLiveQuery> = Vec::new();

    for username in usernames {
        if username.is_empty() {
            continue;
        }

        query.push(UseLiveQuery::new(&username));
    }

    let response: Vec<UseLiveResponse> = match utils::send_query(query).await {
        Ok(data) => data,
        Err(err) => {
            return Err(format!("Failed to fetch UseLive: {err}"));
        }
    };

    let mut live_now: HashMap<String, LiveNow> = HashMap::new();

    for obj in response {
        if obj.data.user.stream.is_none() {
            continue;
        }

        let stream = obj.data.user.stream.unwrap();
        let username = obj.data.user.login;

        let live = LiveNow {
            username: username.clone(),
            started_at: stream.created_at,
        };

        live_now.insert(username, live);
    }

    Ok(live_now)
}

#[derive(Serialize)]
pub struct User {
    id: String,
    username: String,
    avatar: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<Stream>,
}

#[derive(Serialize, Default)]
pub struct Stream {
    title: String,
    started_at: String,
    game: String,
    boxart: String,
    view_count: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

/// Returns up to date information about a user, including third party emotes. Emotes are saved to the store here.
#[tauri::command]
pub async fn fetch_full_user(app: AppHandle, username: &str) -> Result<User, String> {
    if username.is_empty() {
        return Err(String::from("Username cannot be empty"));
    }

    let query = GraphQLQuery::full_user(username);

    let response: GraphQLResponse = match utils::send_query(query).await {
        Ok(response) => response,
        Err(err) => {
            return Err(format!("Failed to fetch user '{username}': {err}"));
        }
    };

    let Some(user) = response.data.user else {
        return Err(format!("User '{username}' not found"));
    };

    // User is streaming, so fetch the stream information
    let stream = match user.stream {
        Some(stream) => {
            let (game_id, game_name) = if let Some(game) = stream.game {
                (game.id, game.name)
            } else {
                (String::new(), String::new())
            };

            let boxart = utils::fetch_game_boxart(game_id).await;

            let view_count = stream.viewers_count.to_string();

            let stream = Stream {
                title: stream.title,
                started_at: stream.created_at,
                game: game_name,
                view_count,
                boxart,
                url: None,
            };

            Some(stream)
        }
        None => None,
    };

    let mut user_emotes: HashMap<String, Emote> = HashMap::new();
    for product in user.subscription_products.unwrap() {
        for emote in product.emotes {
            let name = emote.token;
            let url = format!("{TWITCH_EMOTES_CDN}/{}/default/dark/1.0", emote.id);

            let emote = Emote {
                name: name.clone(),
                url,
                width: 28,
                height: 28,
            };

            user_emotes.insert(name, emote);
        }
    }

    let user_emotes_store = app.store("user_emotes.json").unwrap();
    user_emotes_store.set(username, serde_json::to_value(&user_emotes).unwrap());

    let user_id = user.id.unwrap();

    let seventv_emotes = match emote::fetch_7tv_emotes(&user_id).await {
        Ok(emotes) => emotes,
        Err(err) => {
            error!("Failed to fetch 7tv emotes: {err}");
            HashMap::new()
        }
    };

    let seventv_emotes_store = app.store("seventv_emotes.json").unwrap();
    seventv_emotes_store.set(username, serde_json::to_value(&seventv_emotes).unwrap());

    let bettertv_emotes = match emote::fetch_bettertv_emotes(&user_id).await {
        Ok(emotes) => emotes,
        Err(err) => {
            error!("Failed to fetch bettertv emotes: {err}");
            HashMap::new()
        }
    };

    let bettertv_emotes_store = app.store("bettertv_emotes.json").unwrap();
    bettertv_emotes_store.set(username, serde_json::to_value(&bettertv_emotes).unwrap());

    user_emotes.extend(seventv_emotes);
    user_emotes.extend(bettertv_emotes);

    EMOTES_CACHE
        .lock()
        .unwrap()
        .insert(username.to_string(), user_emotes);

    let user = User {
        id: user_id,
        username: username.to_string(),
        avatar: user.profile_image_url.unwrap(),
        stream,
    };

    Ok(user)
}

#[tauri::command]
pub async fn fetch_stream_info(username: &str, joining_stream: bool) -> Result<Stream, String> {
    if username.is_empty() {
        return Err(String::from("Username cannot be empty"));
    }

    let query = GraphQLQuery::stream_query(username, joining_stream);

    let response: GraphQLResponse = match utils::send_query(query).await {
        Ok(response) => response,
        Err(err) => return Err(format!("Failed to fetch stream info: {err}")),
    };

    let Some(user) = response.data.user else {
        return Err(format!("User '{username}' not found"));
    };

    let Some(stream) = user.stream else {
        return Ok(Stream::default());
    };

    let title = stream.title;
    let started_at = stream.created_at;
    let view_count = stream.viewers_count.to_string();

    let (game_id, game_name) = if let Some(game) = stream.game {
        (game.id, game.name)
    } else {
        (String::new(), String::new())
    };

    let boxart = utils::fetch_game_boxart(game_id).await;

    let url = if joining_stream {
        let Some(stream_playback) = response.data.stream_playback_access_token else {
            return Err(format!(
                "No streamPlaybackAccessToken for '{username}' found"
            ));
        };

        let signature = stream_playback.signature;
        let value = stream_playback.value;

        match playlist_url(username, false, &signature, &value) {
            Ok(url) => Some(url),
            Err(err) => {
                error!("Failed to create playlist URL: {err}");
                None
            }
        }
    } else {
        None
    };

    let stream_info = Stream {
        title,
        started_at,
        game: game_name,
        boxart,
        view_count,
        url,
    };

    Ok(stream_info)
}

#[tauri::command]
pub async fn fetch_stream_playback(username: &str, backup: bool) -> Result<String, String> {
    if username.is_empty() {
        return Err(String::from("No username provided"));
    }

    let query = GraphQLQuery::playback_query(username, backup);

    let response: GraphQLResponse = match utils::send_query(query).await {
        Ok(response) => response,
        Err(err) => {
            return Err(format!("Failed to fetch stream info: {err}"));
        }
    };

    let Some(stream_playback) = response.data.stream_playback_access_token else {
        return Err(String::from("No stream playback access token found"));
    };

    let signature = stream_playback.signature;
    let value = stream_playback.value;

    let url = match playlist_url(username, backup, &signature, &value) {
        Ok(url) => url,
        Err(err) => {
            return Err(format!("Failed to generate playlist URL: {err}"));
        }
    };

    Ok(url)
}

fn playlist_url(username: &str, backup: bool, signature: &str, token: &str) -> Result<String> {
    let mut url = Url::from_str(&format!("http://{LOCAL_API_ADDR}/proxy"))?;
    let mut to_proxy = format!("{USHER_API}/{username}.m3u8");

    let random_number = utils::random_number(1_000_000, 10_000_000);

    if backup {
        to_proxy.push_str(&format!("?platform=ios&supported_codecs=h264&player=twitchweb&fast_bread=true&p={random_number}&sig={signature}&token={token}"));
    } else {
        to_proxy.push_str(&format!("?platform=web&supported_codecs=av1,h265,h264&allow_source=true&player=twitchweb&fast_bread=true&p={random_number}&sig={signature}&token={token}"));
    }

    url.query_pairs_mut()
        .append_pair("username", username)
        .append_pair("url", to_proxy.as_str());

    Ok(url.to_string())
}
