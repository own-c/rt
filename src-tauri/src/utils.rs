use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use axum::http::{HeaderMap, HeaderValue};
use log::error;
use serde::{de::DeserializeOwned, Serialize};

use crate::{user::BOXART_CDN, GRAPHQL_API, HTTP_CLIENT};

pub fn random_number(start: u32, end: u32) -> u32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .subsec_nanos();

    start + (nanos % (end - start))
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

pub async fn send_query<RequestJson, ResponseJson>(body: RequestJson) -> Result<ResponseJson>
where
    RequestJson: Serialize,
    ResponseJson: DeserializeOwned,
{
    let mut headers = HeaderMap::new();

    headers.insert(
        "Client-ID",
        HeaderValue::from_str("kimne78kx3ncx6brgo4mv6wki5h1ko")?,
    );

    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let response = HTTP_CLIENT
        .post(GRAPHQL_API)
        .headers(headers)
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
