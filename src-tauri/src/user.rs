use std::str::FromStr;

use anyhow::{anyhow, Result};
use log::{error, info};
use serde::Serialize;
use serde_json::json;
use tauri::Url;

use crate::{
    api::{self, LOCAL_API},
    emote, utils,
};

const USHER_API: &str = "https://usher.ttvnw.net/api/channel/hls";

const USELIVE_QUERY_HASH: &str = "639d5f11bfb8bf3053b424d9ef650d04c4ebb7d94711d644afb08fe9a0fad5d9";
const COMSCORESTREAMING_QUERY_HASH: &str =
    "e1edae8122517d013405f237ffcc124515dc6ded82480a88daef69c83b53ac01";

pub async fn get_live_now(usernames: Vec<String>) -> Result<Vec<String>> {
    if usernames.is_empty() {
        return Err(anyhow!("No usernames provided"));
    }

    let mut body = json!([]);
    let arr = body.as_array_mut().unwrap();

    for username in usernames {
        if username.is_empty() {
            continue;
        }

        arr.push(json!({
            "operationName": "UseLive",
            "variables": {"channelLogin": username},
            "extensions": {
                "persistedQuery": {
                    "version": 1,
                    "sha256Hash": USELIVE_QUERY_HASH
                }
            }
        }));
    }

    let uselive_query_data = match api::send_gql(body).await {
        Ok(data) => data,
        Err(err) => {
            return Err(anyhow!("Failed to fetch UseLive: {err}"));
        }
    };

    let Some(arr) = uselive_query_data.as_array() else {
        return Err(anyhow!("UseLive data was not an array"));
    };

    let mut live = Vec::new();

    for obj in arr {
        let username = match obj.pointer("/data/user") {
            Some(val) => {
                if val.is_null() {
                    continue;
                }

                match val.get("stream") {
                    Some(val) => {
                        if val.is_null() {
                            continue;
                        }
                    }
                    None => continue,
                };

                let Some(login) = val.get("login") else {
                    continue;
                };

                login.as_str().unwrap_or("")
            }
            None => {
                continue;
            }
        };

        if username.is_empty() {
            continue;
        }

        live.push(username.to_string());
    }

    info!("Live now: {live:?}");
    Ok(live)
}

#[derive(Serialize, Debug)]
pub struct User {
    username: String,
    live: bool,
    avatar: String,
    url: String,
}

pub async fn get_user(username: String) -> Result<User> {
    if username.is_empty() {
        return Err(anyhow!("No username provided"));
    }

    let query = json!({
        "operationName": "ComscoreStreamingQuery",
        "variables": {
            "channel": username,
            "clipSlug": "",
            "isClip": false,
            "isLive": true,
            "isVodOrCollection": false,
            "vodID": ""
        },
        "extensions": {
            "persistedQuery": {
                "version": 1,
                "sha256Hash": COMSCORESTREAMING_QUERY_HASH
            }
        }
    });

    let response = match api::send_gql(query).await {
        Ok(data) => data,
        Err(err) => {
            return Err(anyhow!("Failed to fetch ComscoreStreamingQuery: {err}"));
        }
    };

    let Some(user) = response.pointer("/data/user") else {
        return Err(anyhow!("Missing user in streaming data"));
    };

    let is_streaming = !utils::extract_json(user, "stream")?.is_null();

    // Checking here just to avoid a request for playback tokens if the user isn't streaming
    if !is_streaming {
        let user = User {
            username: username.clone(),
            avatar: String::default(),
            live: false,
            url: String::default(),
        };

        info!("Stream: {{ username: \"{username}\", live: \"false\" }}");

        return Ok(user);
    }

    let stream = fetch_stream(&username, false).await?;

    if let Err(err) = emote::get_user_emotes(&username, &stream.user_id).await {
        error!("Failed to get emotes for '{username}': {err}");
    }

    let user = User {
        username: username.clone(),
        avatar: stream.avatar,
        live: true,
        url: stream.url,
    };

    info!("Stream: {{ username: \"{username}\", live: \"true\" }}");

    Ok(user)
}

pub struct PlaybackResponse {
    user_id: String,
    avatar: String,
    pub url: String,
}

pub async fn fetch_stream(username: &str, backup: bool) -> Result<PlaybackResponse> {
    if username.is_empty() {
        return Err(anyhow!("No username provided"));
    }

    let platform = if backup { "ios" } else { "web" };
    let player_type = if backup { "autoplay" } else { "site" };

    let playback_query = format!(
        r#"{{
            user(login: "{username}") {{
                id
                profileImageURL(width: 50)
            }}
            streamPlaybackAccessToken(
                channelName: "{username}",
                params: {{
                    platform: "{platform}",
                    playerBackend: "mediaplayer",
                    playerType: "{player_type}",
                }}
            )
            {{
                value
                signature
            }}
        }}"#
    );

    let query = json!({"query": playback_query.replace(' ',"")});

    let response = match api::send_gql(query).await {
        Ok(data) => data,
        Err(err) => {
            return Err(anyhow!("Failed to fetch ComscoreStreamingQuery: {err}"));
        }
    };

    let access_token = utils::extract_json(&response, "data")?;
    let access_token_user = utils::extract_json(&access_token, "user")?;

    let user_id = utils::string_from_value(access_token_user.get("id"));

    let avatar = utils::string_from_value(access_token_user.get("profileImageURL"));
    let tokens = utils::extract_json(&access_token, "streamPlaybackAccessToken")?;

    let url = match playlist_url(
        username,
        backup,
        &utils::string_from_value(tokens.get("signature")),
        &utils::string_from_value(tokens.get("value")),
    ) {
        Ok(url) => url.to_string(),
        Err(err) => {
            return Err(anyhow!("Failed to create playlist URL: {err}"));
        }
    };

    let stream = PlaybackResponse {
        user_id,
        avatar,
        url,
    };

    Ok(stream)
}

fn playlist_url(username: &str, backup: bool, signature: &str, token: &str) -> Result<String> {
    let mut url = Url::from_str(&format!("{LOCAL_API}/proxy"))?;
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
