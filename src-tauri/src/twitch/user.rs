use std::{collections::HashMap, sync::atomic::Ordering};

use anyhow::Result;
use log::error;
use serde::Serialize;

use crate::utils;

use super::{
    emote::{self, Emote, TWITCH_EMOTES_CDN},
    main::{self, USHER_API},
    proxy::{BACKUP_STREAM_URL, MAIN_STREAM_URL, USING_BACKUP},
    queries::{GraphQLQuery, GraphQLResponse, UseLiveQuery, UseLiveResponse},
};

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

    let response: Vec<UseLiveResponse> = match main::send_query(query).await {
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
pub async fn fetch_full_user(username: &str) -> Result<User, String> {
    if username.is_empty() {
        return Err(String::from("Username cannot be empty"));
    }

    let query = GraphQLQuery::full_user(username);

    let response: GraphQLResponse = match main::send_query(query).await {
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

            let boxart = main::fetch_game_boxart(game_id).await;

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

    let user_id = user.id.unwrap();

    let seventv_emotes = match emote::fetch_7tv_emotes(&user_id).await {
        Ok(emotes) => emotes,
        Err(err) => {
            error!("Failed to fetch 7tv emotes: {err}");
            HashMap::new()
        }
    };

    let bettertv_emotes = match emote::fetch_bettertv_emotes(&user_id).await {
        Ok(emotes) => emotes,
        Err(err) => {
            error!("Failed to fetch bettertv emotes: {err}");
            HashMap::new()
        }
    };

    user_emotes.extend(seventv_emotes);
    user_emotes.extend(bettertv_emotes);

    if let Err(err) = emote::update_user_emotes(username, user_emotes).await {
        error!("Failed to save emotes for user '{username}': {err}");
    }

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

    let response: GraphQLResponse = match main::send_query(query).await {
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

    let boxart = main::fetch_game_boxart(game_id).await;

    let url = if joining_stream {
        let Some(stream_playback) = response.data.stream_playback_access_token else {
            return Err(format!(
                "No streamPlaybackAccessToken for '{username}' found"
            ));
        };

        let signature = stream_playback.signature;
        let value = stream_playback.value;

        let url = playlist_url(username, false, &signature, &value);

        USING_BACKUP.store(false, Ordering::Relaxed);
        *MAIN_STREAM_URL.lock().await = None;
        *BACKUP_STREAM_URL.lock().await = None;

        Some(url)
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

    let response: GraphQLResponse = match main::send_query(query).await {
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

    let url = playlist_url(username, backup, &signature, &value);

    Ok(url)
}

fn playlist_url(username: &str, backup: bool, signature: &str, token: &str) -> String {
    let mut url = format!("{USHER_API}/{username}.m3u8");

    let random_number = utils::random_number(1_000_000, 10_000_000);

    if backup {
        url.push_str(&format!("?platform=ios&supported_codecs=h264&player=twitchweb&fast_bread=true&p={random_number}&sig={signature}&token={token}"));
    } else {
        url.push_str(&format!("?platform=web&supported_codecs=av1,h265,h264&allow_source=true&player=twitchweb&fast_bread=true&p={random_number}&sig={signature}&token={token}"));
    }

    url.to_string()
}
