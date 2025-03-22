use std::{path::Path, time::Duration};

use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use log::error;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::SqlitePool;
use tauri::async_runtime::{self, Mutex};
use tauri_plugin_http::reqwest::{
    header::{HeaderMap, HeaderValue},
    redirect::Policy,
    Client,
};

pub const USHER_API: &str = "https://usher.ttvnw.net/api/channel/hls";
const GRAPHQL_API: &str = "https://gql.twitch.tv/gql";
const BOXART_CDN: &str = "https://static-cdn.jtvnw.net/ttv-boxart";

pub const CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";

lazy_static! {
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
    async_runtime::block_on(async {
        let emotes_db_path = app_data_path.join("emotes.db");
        let emotes_db = SqlitePool::connect(emotes_db_path.to_str().unwrap()).await?;

        *EMOTES_DB.lock().await = Some(emotes_db);

        Ok(())
    })
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
