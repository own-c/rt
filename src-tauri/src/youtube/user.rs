use anyhow::Result;

use super::main::RP_CLIENT;

pub struct YouTubeUser {
    pub id: String,
    pub username: String,
    pub avatar: String,
}

pub async fn fetch_user(username: &str) -> Result<YouTubeUser> {
    let client = RP_CLIENT.lock().await;

    let url_target = client.query().resolve_string(username, false).await?;
    let url = url_target.to_url();
    let channel_id = url.split('/').last().unwrap();

    let channel = client.query().channel_videos(channel_id).await?;

    let avatar = match channel.avatar.first() {
        Some(avatar) => avatar.url.clone(),
        None => String::new(),
    };

    let user = YouTubeUser {
        id: channel.id,
        username: channel.name,
        avatar,
    };

    Ok(user)
}
