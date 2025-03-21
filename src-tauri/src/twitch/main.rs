use std::{path::Path, time::Duration};

use anyhow::{anyhow, Context, Result};
use axum::{
    http::{HeaderMap, HeaderValue},
    routing::get,
    Router,
};
use lazy_static::lazy_static;
use log::{error, info};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::SqlitePool;
use tauri::async_runtime;
use tauri_plugin_http::reqwest::{redirect::Policy, Client};
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc, Mutex},
};
use tower_http::cors::{Any, CorsLayer};

use crate::twitch::{chat, proxy};

pub const LOCAL_API_ADDR: &str = "127.0.0.1:3030";

pub const USHER_API: &str = "https://usher.ttvnw.net/api/channel/hls";
const GRAPHQL_API: &str = "https://gql.twitch.tv/gql";
const BOXART_CDN: &str = "https://static-cdn.jtvnw.net/ttv-boxart";

pub const CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";

pub struct ChatState {
    pub current_chat: Option<String>,
    /// For sending messages to the chat websocket.
    pub sender: mpsc::Sender<String>,
    /// For receiving messages from the websocket.
    pub receiver: broadcast::Sender<String>,
}

lazy_static! {
    pub static ref CHAT_STATE: Mutex<Option<ChatState>> = Mutex::new(None);

    pub static ref EMOTES_DB: Mutex<Option<SqlitePool>> = Mutex::new(None);

    pub static ref HTTP_CLIENT: Client = {
        let mut headers = HeaderMap::new();
        headers.insert("Client-ID", HeaderValue::from_static(CLIENT_ID));
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        Client::builder()
            .use_rustls_tls()
            .https_only(true)
            .http2_prior_knowledge()
            .default_headers(headers)
            .redirect(Policy::none())
            .gzip(true)
            .build()
            .unwrap()
    };

    // Specifically used in the proxy.
    pub static ref PROXY_HTTP_CLIENT: Client = Client::builder()
       .use_rustls_tls()
       .https_only(true)
       .tcp_keepalive(Duration::from_secs(5))
       .gzip(true)
       .build()
       .unwrap();
}

pub fn setup(app_data_path: &Path) -> Result<()> {
    async_runtime::spawn(async { start_api_server().await });

    async_runtime::block_on(async {
        let emotes_db_path = app_data_path.join("emotes.db");
        let emotes_db = SqlitePool::connect(emotes_db_path.to_str().unwrap()).await?;

        *EMOTES_DB.lock().await = Some(emotes_db);

        init_irc_chat().await
    })?;

    Ok(())
}

async fn init_irc_chat() -> Result<()> {
    let (ws_sender, ws_receiver) = chat::init_irc_connection().await?;

    let chat = Some(ChatState {
        current_chat: None,
        sender: ws_sender,
        receiver: ws_receiver,
    });

    *CHAT_STATE.lock().await = chat;

    info!("Initialized chat state");
    Ok(())
}

async fn start_api_server() -> Result<()> {
    let cors_layer = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    let listener = TcpListener::bind(LOCAL_API_ADDR).await?;

    let app = Router::new()
        .route("/chat/{username}", get(chat::join_chat))
        .route("/proxy", get(proxy::proxy_stream))
        .layer(cors_layer);

    info!("Starting API server");
    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn send_query<RequestJson, ResponseJson>(body: RequestJson) -> Result<ResponseJson>
where
    RequestJson: Serialize,
    ResponseJson: DeserializeOwned,
{
    let response = HTTP_CLIENT
        .post(GRAPHQL_API)
        .json(&body)
        .send()
        .await
        .context("Failed to send GraphQL request")?;

    let status = response.status();

    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .context("Failed to read GraphQL response")?;

        return Err(anyhow!("GraphQL request failed: {status} - {error_body}"));
    }

    response
        .json()
        .await
        .context("Failed to deserialize GraphQL response")
}

pub async fn fetch_game_boxart(game_id: String) -> String {
    let mut boxart = String::from("https://static-cdn.jtvnw.net/ttv-static/404_boxart-144x192.jpg");
    if game_id.is_empty() {
        return boxart;
    }

    // Twitch usually has an updated box art in URLs without the _IGDB tag, try to use it if it exists
    match HTTP_CLIENT
        .get(format!("{BOXART_CDN}/{game_id}-144x192.jpg"))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                boxart = format!("{BOXART_CDN}/{game_id}-144x192.jpg").to_string();
            } else {
                boxart = format!("{BOXART_CDN}/{game_id}_IGDB-144x192.jpg").to_string();
            };
        }
        Err(err) => {
            error!("Failed to fetch game box art: {err}");
        }
    }

    boxart
}
