use anyhow::Result;
use serde::Serialize;

use super::main::RP_CLIENT;

#[derive(Serialize)]
pub struct YouTubeVideo {
    pub id: String,
    pub username: String,
    pub title: String,
    pub thumbnail: String,
    pub publish_date: String,
    pub view_count: String,
}

pub async fn fetch_videos(channel_ids: Vec<String>) -> Result<Vec<YouTubeVideo>> {
    let mut videos = Vec::new();

    let client = RP_CLIENT.lock().await;

    for channel_id in channel_ids {
        let channel = client.query().channel_rss(channel_id).await?;

        videos.reserve_exact(channel.videos.len());

        channel.videos.iter().for_each(|video| {
            let video = YouTubeVideo {
                id: video.id.clone(),
                username: channel.name.clone(),
                title: video.name.clone(),
                thumbnail: video.thumbnail.url.clone(),
                publish_date: video.publish_date.unix_timestamp().to_string(),
                view_count: video.view_count.to_string(),
            };

            videos.push(video);
        });
    }

    Ok(videos)
}
