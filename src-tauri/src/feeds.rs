use anyhow::Result;
use serde::Serialize;
use sqlx::Row;
use tauri::{async_runtime::Mutex, AppHandle, Emitter, State};

use crate::{
    twitch::{self, live::LiveNow},
    users::Platform,
    AppState,
};

#[derive(Serialize)]
pub struct Feed {
    twitch: Option<Vec<LiveNow>>,
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

        return Ok(Feed { twitch: Some(feed) });
    }

    Err(format!("Invalid platform '{platform}'"))
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

        let live_now = match twitch::live::fetch_live_now(usernames).await {
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
    }

    if let Err(err) = app_handle.emit("update_view", platform) {
        return Err(format!("Error emitting 'update_view' event: {err}"));
    }

    Ok(())
}
