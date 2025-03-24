use anyhow::Result;
use serde::Serialize;
use sqlx::Row;
use tauri::{async_runtime::Mutex, AppHandle, Emitter, State};

use crate::{
    twitch::{self, stream::LiveNow},
    user::Platform,
    youtube::{self, video::YouTubeVideo},
    AppState,
};

#[derive(Serialize)]
pub struct Feed {
    twitch: Option<Vec<LiveNow>>,
    youtube: Option<Vec<YouTubeVideo>>,
}

#[tauri::command]
pub async fn get_feed(
    state: State<'_, Mutex<AppState>>,
    platform: Platform,
) -> Result<Feed, String> {
    let state = state.lock().await;
    let feeds_db = state.feeds_db.as_ref().unwrap();

    if platform == Platform::Twitch {
        let query = "SELECT username, started_at FROM twitch";

        let rows = match sqlx::query(query).fetch_all(feeds_db).await {
            Ok(rows) => rows,
            Err(err) => {
                return Err(format!("Failed to fetch feed: {err}"));
            }
        };

        let mut feed: Vec<LiveNow> = Vec::new();

        for row in rows {
            let live_now = LiveNow {
                username: row.try_get("username").map_err(|e| e.to_string())?,
                started_at: row.try_get("started_at").map_err(|e| e.to_string())?,
            };

            feed.push(live_now);
        }

        return Ok(Feed {
            twitch: Some(feed),
            youtube: None,
        });
    }

    if platform == Platform::YouTube {
        let query = "SELECT id, username, title, thumbnail, published_at, view_count FROM youtube";

        let rows = match sqlx::query(query).fetch_all(feeds_db).await {
            Ok(rows) => rows,
            Err(err) => {
                return Err(format!("Failed to fetch feed: {err}"));
            }
        };

        let mut feed: Vec<YouTubeVideo> = Vec::new();

        for row in rows {
            let video = YouTubeVideo {
                id: row.try_get("id").map_err(|e| e.to_string())?,
                username: row.try_get("username").map_err(|e| e.to_string())?,
                title: row.try_get("title").map_err(|e| e.to_string())?,
                thumbnail: row.try_get("thumbnail").map_err(|e| e.to_string())?,
                publish_date: row.try_get("published_at").map_err(|e| e.to_string())?,
                view_count: row.try_get("view_count").map_err(|e| e.to_string())?,
            };

            feed.push(video);
        }

        return Ok(Feed {
            youtube: Some(feed),
            twitch: None,
        });
    }

    Err(format!("Invalid platform '{platform:#?}'"))
}

#[tauri::command]
pub async fn refresh_feed(
    app_handle: AppHandle,
    state: State<'_, Mutex<AppState>>,
    platform: Platform,
) -> Result<(), String> {
    let state = state.lock().await;
    let users_db = state.users_db.as_ref().unwrap();
    let feeds_db = state.feeds_db.as_ref().unwrap();

    if platform == Platform::Twitch {
        let query = "SELECT username FROM twitch";

        let rows = match sqlx::query(query).fetch_all(users_db).await {
            Ok(rows) => rows,
            Err(err) => {
                return Err(format!("Failed to fetch usernames from database: {err}"));
            }
        };

        let mut usernames: Vec<String> = Vec::new();

        for row in rows {
            let username = row.try_get("username").map_err(|e| e.to_string())?;
            usernames.push(username);
        }

        let live_now = match twitch::stream::fetch_live_now(usernames).await {
            Ok(live_now) => live_now,
            Err(err) => {
                return Err(format!("Failed to fetch live now: {err}"));
            }
        };

        let query = "DELETE FROM twitch";

        sqlx::query(query)
            .execute(feeds_db)
            .await
            .map_err(|e| e.to_string())?;

        for (username, live) in live_now {
            let query = "INSERT INTO twitch (username, started_at) VALUES (?, ?)";

            sqlx::query(query)
                .bind(&username)
                .bind(&live.started_at)
                .execute(feeds_db)
                .await
                .map_err(|e| e.to_string())?;
        }

        if let Err(err) = app_handle.emit("updated_streams", &platform) {
            return Err(format!("Error emitting 'updated_streams' event: {err}"));
        }
    }

    if platform == Platform::YouTube {
        let query = "SELECT id FROM youtube";

        let rows = match sqlx::query(query).fetch_all(users_db).await {
            Ok(rows) => rows,
            Err(err) => {
                return Err(format!("Failed to fetch ids from database: {err}"));
            }
        };

        let mut channel_ids: Vec<String> = Vec::new();

        for row in rows {
            let channel_id = row.try_get("id").map_err(|e| e.to_string())?;
            channel_ids.push(channel_id);
        }

        let videos = match youtube::video::fetch_videos(channel_ids).await {
            Ok(videos) => videos,
            Err(err) => {
                return Err(format!("Failed to fetch videos: {err}"));
            }
        };

        let query = "DELETE FROM youtube";

        sqlx::query(query)
            .execute(feeds_db)
            .await
            .map_err(|e| e.to_string())?;

        for video in videos {
            let query = "INSERT INTO youtube (id, username, title, thumbnail, published_at, view_count) VALUES (?, ?, ?, ?, ?, ?)";

            sqlx::query(query)
                .bind(&video.id)
                .bind(&video.username)
                .bind(&video.title)
                .bind(&video.thumbnail)
                .bind(&video.publish_date)
                .bind(video.view_count)
                .execute(feeds_db)
                .await
                .map_err(|e| e.to_string())?;
        }

        if let Err(err) = app_handle.emit("updated_videos", platform) {
            return Err(format!("Error emitting 'updated_videos' event: {err}"));
        }
    }

    Ok(())
}
