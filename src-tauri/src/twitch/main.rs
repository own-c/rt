use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use serde::{de::DeserializeOwned, Serialize};
use tauri_plugin_http::reqwest::{
    header::{HeaderMap, HeaderValue},
    redirect::Policy,
    Client,
};

const GRAPHQL_API: &str = "https://gql.twitch.tv/gql";

const CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";

lazy_static! {
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
       .tcp_keepalive(Duration::from_secs(10))
       .gzip(true)
       .build()
       .unwrap();
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
