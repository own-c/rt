use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};

use crate::twitch::main::HTTP_CLIENT;

pub fn random_number(start: u32, end: u32) -> u32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .subsec_nanos();

    start + (nanos % (end - start))
}

pub async fn download_image(url: &str) -> Result<Vec<u8>> {
    let response = HTTP_CLIENT.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to download image: {url}"));
    }

    let bytes = response.bytes().await?;

    Ok(bytes.to_vec())
}
