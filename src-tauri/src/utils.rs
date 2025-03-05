use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use serde_json::Value;

pub fn string_from_value(value: Option<&Value>) -> String {
    match value {
        None => String::new(),
        Some(val) => val.as_str().unwrap_or("").to_string(),
    }
}

pub fn extract_json(json: &Value, field: &str) -> Result<Value> {
    json.get(field)
        .ok_or_else(|| anyhow!("No '{field}' field found"))
        .cloned()
}

pub fn random_number(start: u32, end: u32) -> u32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .subsec_nanos();

    start + (nanos % (end - start))
}
