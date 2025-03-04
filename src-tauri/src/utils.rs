use std::time::{SystemTime, UNIX_EPOCH};

use axum::{http::StatusCode, Json};
use log::error;
use serde_json::Value;
use tauri_plugin_http::reqwest::Client;

pub fn string_from_value(value: Option<&Value>) -> String {
    match value {
        None => String::new(),
        Some(val) => val.as_str().unwrap_or("").to_string(),
    }
}

pub fn extract_json_field<'a>(
    json: &'a Value,
    field: &str,
) -> Result<&'a Value, (StatusCode, Json<Value>)> {
    json.get(field).ok_or_else(|| {
        error!("No '{field}' field found");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(Value::Null))
    })
}

pub fn random_number(start: u32, end: u32) -> u32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .subsec_nanos();

    start + (nanos % (end - start))
}

pub fn new_http_client() -> Client {
    Client::builder()
        .gzip(true)
        .use_rustls_tls()
        .https_only(true)
        .http2_prior_knowledge()
        .build()
        .unwrap()
}
