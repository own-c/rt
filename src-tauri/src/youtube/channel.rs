use anyhow::Result;

use crate::{
    user::{Platform, User},
    util,
};

use super::main::RP_CLIENT;

pub async fn fetch_channel_by_name(channel_name: &str) -> Result<User> {
    let client = RP_CLIENT.lock().await;

    let url_target = client.query().resolve_string(channel_name, false).await?;
    let url = url_target.to_url();
    let channel_id = url.split('/').last().unwrap();

    let channel = client.query().channel_videos(channel_id).await?;

    let avatar = match channel.avatar.first() {
        Some(avatar) => util::download_image(&avatar.url).await?,
        None => Vec::new(),
    };

    let user = User {
        id: channel.id,
        username: channel.name,
        avatar,
        platform: Platform::YouTube,
    };

    Ok(user)
}

pub async fn fetch_channel_by_id(channel_id: &str) -> Result<User> {
    let client = RP_CLIENT.lock().await;

    let channel = client.query().channel_videos(channel_id).await?;

    let avatar = match channel.avatar.first() {
        Some(avatar) => util::download_image(&avatar.url).await?,
        None => Vec::new(),
    };

    let user = User {
        id: channel.id,
        username: channel.name,
        avatar,
        platform: Platform::YouTube,
    };

    Ok(user)
}
