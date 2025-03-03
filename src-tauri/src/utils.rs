use axum::{http::StatusCode, Json};
use serde_json::Value;

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
        println!("No '{field}' field found");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(Value::Null))
    })
}
