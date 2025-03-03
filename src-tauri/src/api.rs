use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::Url;
use tauri_plugin_http::reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    emotes::fetch_emotes,
    proxy::proxy,
    utils::{extract_json_field, string_from_value},
};

lazy_static! {
    static ref HTTP_CLIENT: Client = Client::new();
}

const USHER_API: &str = "https://usher.ttvnw.net/api/channel/hls";
const GRAPHQL_API: &str = "https://gql.twitch.tv/gql";
pub const LOCAL_API: &str = "http://127.0.0.1:3030";
const LOCAL_API_ADDR: &str = "127.0.0.1:3030";

// Stream configuration
const CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";

// GraphQL queries hashes
const COMSCORESTREAMING_QUERY_HASH: &str =
    "e1edae8122517d013405f237ffcc124515dc6ded82480a88daef69c83b53ac01";
const USELIVE_QUERY_HASH: &str = "639d5f11bfb8bf3053b424d9ef650d04c4ebb7d94711d644afb08fe9a0fad5d9";

pub async fn start_api_server() -> Result<()> {
    let cors_layer = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    let app = Router::new()
        .route("/proxy", get(proxy))
        .route("/user/{username}", get(get_stream))
        .route("/live", get(get_live_now))
        .layer(cors_layer);

    println!("Binding api server on {}", LOCAL_API_ADDR);
    let listener = TcpListener::bind(LOCAL_API_ADDR).await?;

    println!("Listening on {}", LOCAL_API_ADDR);
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

// API routes

#[derive(Deserialize)]
struct LiveNowQuery {
    usernames: String,
}

async fn get_live_now(usernames: Query<LiveNowQuery>) -> impl IntoResponse {
    let Query(query) = usernames;

    if query.usernames.is_empty() {
        println!("No usernames provided");
        return (StatusCode::BAD_REQUEST, Json(vec![]));
    }

    let mut body = json!([]);
    let arr = body.as_array_mut().unwrap();

    query.usernames.split(",").for_each(|username| {
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

    let uselive_query_data = match send_gql(body).await {
        Ok(data) => data,
        Err(err) => {
            println!("Error fetching UseLive: {err}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]));
        }
    };

    let arr = match uselive_query_data.as_array() {
        Some(arr) => arr,
        None => {
            println!("UseLive data was not an array");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]));
        }
    };

    let mut live = Vec::new();

    for obj in arr {
        let data = match obj.get("data") {
            Some(data) => data,
            None => continue,
        };

        let user = match data.get("user") {
            Some(user) => user,
            None => continue,
        };

        let stream = match user.get("stream") {
            Some(stream) => stream,
            None => continue,
        };

        if stream.is_null() {
            continue;
        }

        let username = string_from_value(user.get("login"));

        if username.is_empty() {
            continue;
        }

        live.push(username);
    }

    println!("Live: {:?}", live);
    (StatusCode::OK, Json(live))
}

#[derive(Serialize, Default)]
struct Stream {
    username: String,
    title: String,
    started_at: String,
    avatar: String,
    game: String,
    url: String,
    emotes: Vec<Emote>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Emote {
    pub name: String,
    pub url: String,
    pub width: i64,
    pub height: i64,
}

async fn get_stream(
    username: Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let Path(username) = username;

    if username.is_empty() {
        println!("No username provided");
        return Ok((StatusCode::BAD_REQUEST, Json(Value::Null)));
    }

    let streaming_query = json!({
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

    let stream_query_data = match send_gql(streaming_query).await {
        Ok(data) => data,
        Err(err) => {
            println!("Error fetching ComscoreStreamingQuery: {err}");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(Value::Null)));
        }
    };

    let data = extract_json_field(&stream_query_data, "data")?;
    let user = extract_json_field(data, "user")?;
    let stream = extract_json_field(user, "stream")?;
    let broadcast_settings = extract_json_field(user, "broadcastSettings")?;
    let game = stream.get("game").unwrap_or(&Value::Null);
    let game = if game.is_null() {
        String::new()
    } else {
        string_from_value(game.get("name"))
    };

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

    let query = json!({"query": playback_query.replace(" ","")});

    let playback_query_data = match send_gql(query).await {
        Ok(data) => data,
        Err(err) => {
            println!("Error fetching streamPlaybackAccessToken: {err}");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::Value::String(err.to_string())),
            ));
        }
    };

    let access_token = extract_json_field(&playback_query_data, "data")?;
    let access_token_user = extract_json_field(access_token, "user")?;
    let id = string_from_value(access_token_user.get("id"));
    let avatar = string_from_value(access_token_user.get("profileImageURL"));

    let playback = extract_json_field(access_token, "streamPlaybackAccessToken")?;

    let playlist_url = match playlist_url(
        &username,
        &string_from_value(playback.get("signature")),
        &string_from_value(playback.get("value")),
    ) {
        Ok(url) => url.to_string(),
        Err(err) => {
            println!("Error creating playlist URL: {err}");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::Value::String(err.to_string())),
            ));
        }
    };

    let emotes = fetch_emotes(&id).await;

    let stream = Stream {
        username,
        title: string_from_value(broadcast_settings.get("title")),
        started_at: string_from_value(stream.get("createdAt")),
        avatar,
        url: playlist_url,
        game,
        emotes,
    };

    let stream = serde_json::to_value(stream).unwrap();
    Ok((StatusCode::OK, Json(stream)))
}

fn playlist_url(channel_name: &str, signature: &str, token: &str) -> Result<Url> {
    let mut url = Url::from_str(&format!("{LOCAL_API}/proxy"))?;
    let mut to_proxy = Url::from_str(&format!("{USHER_API}/{channel_name}.m3u8"))?;

    let start = 1_000_000;
    let end = 10_000_000;

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .subsec_nanos();

    let random_number = start + (nanos % (end - start));

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

pub async fn send_gql(ops: Value) -> Result<Value> {
    let mut headers = HeaderMap::new();
    headers.insert("Client-ID", HeaderValue::from_str(CLIENT_ID)?);
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/plain;charset=UTF-8"),
    );

    let req = HTTP_CLIENT
        .post(GRAPHQL_API)
        .headers(headers)
        .body(ops.to_string())
        .build()?;

    let resp = HTTP_CLIENT.execute(req).await?.bytes().await?;
    Ok(serde_json::from_slice::<Value>(&resp)?)
}
