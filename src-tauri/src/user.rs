use std::{collections::HashMap, str::FromStr};

use anyhow::Result;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use lazy_static::lazy_static;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{async_runtime::Mutex, Url};
use tauri_plugin_http::reqwest::Client;

use crate::{
    api::{self, LOCAL_API},
    utils,
};

const USHER_API: &str = "https://usher.ttvnw.net/api/channel/hls";

const USELIVE_QUERY_HASH: &str = "639d5f11bfb8bf3053b424d9ef650d04c4ebb7d94711d644afb08fe9a0fad5d9";
const COMSCORESTREAMING_QUERY_HASH: &str =
    "e1edae8122517d013405f237ffcc124515dc6ded82480a88daef69c83b53ac01";

#[derive(Deserialize)]
pub struct LiveNowQuery {
    usernames: String,
}

#[derive(Serialize)]
struct Stream {
    username: String,
    live: bool,
    avatar: String,
    url: String,
}

lazy_static! {
    static ref USER_HTTP_CLIENT: Client = utils::new_http_client();
    pub static ref USER_TO_ID: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub async fn get_live_now(usernames: Query<LiveNowQuery>) -> impl IntoResponse {
    let Query(query) = usernames;

    if query.usernames.is_empty() {
        error!("No usernames provided");
        return (StatusCode::BAD_REQUEST, Json(vec![]));
    }

    let mut body = json!([]);
    let arr = body.as_array_mut().unwrap();

    query.usernames.split(',').for_each(|username| {
        if username.is_empty() {
            return;
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
    });

    let uselive_query_data = match api::send_gql(body).await {
        Ok(data) => data,
        Err(err) => {
            error!("Fetching UseLive: {err}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]));
        }
    };

    let Some(arr) = uselive_query_data.as_array() else {
        error!("UseLive data was not an array");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]));
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

    info!("Live now: {:?}", live);
    (StatusCode::OK, Json(live))
}

pub async fn get_user_stream(
    username: Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let Path(username) = username;

    if username.is_empty() {
        error!("No username provided");
        return Ok((StatusCode::BAD_REQUEST, Json(Value::Null)));
    }

    let mut query = json!([{
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
    }]);

    let playback_query = format!(
        r#"{{
            user(login: "{username}") {{
                id
                profileImageURL(width: 50)
            }}
            streamPlaybackAccessToken(
                channelName: "{username}",
                params: {{
                    platform: "web",
                    playerBackend: "mediaplayer",
                    playerType: "site"
                }}
            )
            {{
                value
                signature
            }}
        }}"#
    );

    query
        .as_array_mut()
        .unwrap()
        .push(json!({"query": playback_query.replace(' ',"")}));

    let response = match api::send_gql(query).await {
        Ok(data) => data,
        Err(err) => {
            error!("Fetching ComscoreStreamingQuery: {err}");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(Value::Null)));
        }
    };

    let response = response.as_array();

    if response.is_none() || response.unwrap().len() != 2 {
        error!(
            "Missing data in query: {}",
            serde_json::to_string(&response).unwrap()
        );
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(Value::Null)));
    }

    let response = response.unwrap();
    let streaming = response.first().unwrap();

    let Some(user) = streaming.pointer("/data/user") else {
        error!("Missing user in streaming data");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(Value::Null)));
    };

    let stream = utils::extract_json_field(user, "stream")?;

    let playback = response.last().unwrap();

    let access_token = utils::extract_json_field(playback, "data")?;
    let access_token_user = utils::extract_json_field(access_token, "user")?;

    let id = utils::string_from_value(access_token_user.get("id"));
    let mut lock = USER_TO_ID.lock().await;
    lock.insert(username.clone(), id.clone());

    let avatar = utils::string_from_value(access_token_user.get("profileImageURL"));

    let playback_tokens = utils::extract_json_field(access_token, "streamPlaybackAccessToken")?;

    let playlist_url = match playlist_url(
        &username,
        &utils::string_from_value(playback_tokens.get("signature")),
        &utils::string_from_value(playback_tokens.get("value")),
    ) {
        Ok(url) => url.to_string(),
        Err(err) => {
            error!("Creating playlist URL: {err}");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::Value::String(err.to_string())),
            ));
        }
    };

    let stream = Stream {
        username: username.clone(),
        avatar,
        live: !stream.is_null(),
        url: playlist_url,
    };

    let stream = serde_json::to_value(stream).unwrap();
    Ok((StatusCode::OK, Json(stream)))
}

fn playlist_url(channel_name: &str, signature: &str, token: &str) -> Result<Url> {
    let mut url = Url::from_str(&format!("{LOCAL_API}/proxy"))?;
    let mut to_proxy = Url::from_str(&format!("{USHER_API}/{channel_name}.m3u8"))?;

    let random_number = utils::random_number(1_000_000, 10_000_000);

    to_proxy
        .query_pairs_mut()
        .append_pair("allow_source", "true")
        .append_pair("p", &random_number.to_string())
        .append_pair("platform", "web")
        .append_pair("player", "twitchweb")
        .append_pair("supported_codecs", "av1,h265,h264")
        .append_pair("sig", signature)
        .append_pair("token", token);

    url.query_pairs_mut().append_pair("url", to_proxy.as_ref());

    Ok(url)
}
