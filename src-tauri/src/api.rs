use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use async_runtime::Mutex;
use axum::{routing::get, Router};
use lazy_static::lazy_static;
use log::info;
use serde_json::Value;
use tauri::async_runtime;
use tauri_plugin_http::reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::{chat, proxy};

const GRAPHQL_API: &str = "https://gql.twitch.tv/gql";

pub const LOCAL_API: &str = "http://127.0.0.1:3030";
const LOCAL_API_ADDR: &str = "127.0.0.1:3030";

// Stream configuration
const CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";

lazy_static! {
    pub static ref HTTP_CLIENT: Client = Client::builder()
        .gzip(true)
        .use_rustls_tls()
        .https_only(true)
        .http2_prior_knowledge()
        .build()
        .unwrap();
}

pub async fn start_api_server() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let cors_layer = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    info!("Binding API server on {}", LOCAL_API_ADDR);
    let listener = TcpListener::bind(LOCAL_API_ADDR).await?;

    let ws_stream = chat::init_irc_connection().await?;

    let app = Router::new()
        .route("/proxy", get(proxy::proxy_stream))
        .route("/chat/{username}", get(chat::join_chat))
        .with_state(Arc::new(Mutex::new(ws_stream)))
        .layer(cors_layer);

    axum::serve(listener, app).await.unwrap();
    Ok(())
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
        .build()
        .context("Failed to build GraphQL request")?;

    let resp = HTTP_CLIENT
        .execute(req)
        .await
        .context("Failed to execute GraphQL request")?;

    let status = resp.status();

    if !status.is_success() {
        let error_body = resp
            .text()
            .await
            .unwrap_or_else(|err| format!("Failed to fetch error body: {err}"));

        return Err(anyhow!(
            "GraphQL request failed: {status} - Response: {error_body}"
        ));
    }

    let body = resp
        .bytes()
        .await
        .context("Failed to read GraphQL response body")?;

    serde_json::from_slice::<Value>(&body).context("Failed to deserialize GraphQL response")
}
