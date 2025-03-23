use std::fmt::Display;

use anyhow::Result;

use log::error;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::Type, Pool, Row, Sqlite};
use tauri::{async_runtime::Mutex, AppHandle, Emitter, State};

use crate::{twitch, utils, AppState};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub platform: Platform,
    #[serde(rename = "avatarBlob")]
    pub avatar_blob: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Type)]
pub enum Platform {
    #[serde(rename = "youtube")]
    Youtube,
    #[serde(rename = "twitch")]
    Twitch,
}

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Youtube => write!(f, "youtube"),
            Platform::Twitch => write!(f, "twitch"),
        }
    }
}

#[tauri::command]
pub async fn get_users(
    state: State<'_, Mutex<AppState>>,
    platform: Option<Platform>,
) -> Result<Vec<User>, String> {
    let state = state.lock().await;
    let users_db = state.users_db.as_ref().unwrap();

    let users = match platform {
        Some(platform) => get_users_for_platform(users_db, platform).await,
        None => {
            // TODO: Merge users from all platforms when adding Youtube
            get_users_for_platform(users_db, Platform::Twitch).await
        }
    };

    if let Err(err) = users {
        return Err(format!("Failed to get users: {err}"));
    }

    Ok(users.unwrap())
}

#[tauri::command]
pub async fn add_user(
    app_handle: AppHandle,
    state: State<'_, Mutex<AppState>>,
    username: String,
    platform: Platform,
) -> Result<(), String> {
    let state = state.lock().await;
    let users_db = state.users_db.as_ref().unwrap();
    let emotes_db = state.emotes_db.as_ref().unwrap();

    if platform == Platform::Twitch {
        let user = match twitch::user::fetch_user(&username).await {
            Ok(user) => user,
            Err(err) => {
                return Err(format!("Failed to fetch user '{username}': {err}"));
            }
        };

        if let Err(err) = twitch::emote::update_user_emotes(emotes_db, &username, user.emotes).await
        {
            error!("Failed to save emotes for user '{username}': {err}");
        }

        let avatar = match utils::download_image(&user.avatar).await {
            Ok(bytes) => bytes,
            Err(err) => {
                return Err(format!(
                    "Failed to download avatar for user '{username}': {err}"
                ));
            }
        };

        let query = "INSERT INTO twitch (id, username, avatar) VALUES (?, ?, ?) ON CONFLICT (username) DO UPDATE SET avatar = ?";

        sqlx::query(query)
            .bind(&user.id)
            .bind(&user.username)
            .bind(&avatar)
            .bind(&avatar)
            .execute(users_db)
            .await
            .map_err(|e| e.to_string())?;

        if let Err(err) = app_handle.emit("update_view", platform) {
            error!("Failed to emit 'update_view' event: {err}");
        }

        return Ok(());
    }

    Err(format!("Invalid platform '{platform}'"))
}

#[tauri::command]
pub async fn remove_user(
    app_handle: AppHandle,
    state: State<'_, Mutex<AppState>>,
    username: String,
    platform: Platform,
) -> Result<(), String> {
    let state = state.lock().await;
    let users_db = state.users_db.as_ref().unwrap();
    let feeds_db = state.feeds_db.as_ref().unwrap();
    let emotes_db = state.emotes_db.as_ref().unwrap();

    if platform == Platform::Twitch {
        let query = "DELETE FROM twitch WHERE username = ?";

        sqlx::query(query)
            .bind(&username)
            .execute(users_db)
            .await
            .map_err(|e| e.to_string())?;

        let query = "DELETE FROM twitch WHERE username = ?";

        sqlx::query(query)
            .bind(&username)
            .execute(feeds_db)
            .await
            .map_err(|e| e.to_string())?;

        let query = "DELETE FROM twitch WHERE username = ?";

        sqlx::query(query)
            .bind(&username)
            .execute(emotes_db)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Err(err) = app_handle.emit("update_view", platform) {
        return Err(format!("Error emitting 'update_view' event: {err}"));
    }

    Ok(())
}

async fn get_users_for_platform(users_db: &Pool<Sqlite>, platform: Platform) -> Result<Vec<User>> {
    let query = format!("SELECT username, avatar FROM {platform}");

    let rows = sqlx::query(&query).fetch_all(users_db).await?;

    let mut users: Vec<User> = Vec::with_capacity(rows.len());

    for row in rows {
        let user = User {
            username: row.try_get("username")?,
            avatar_blob: row.try_get("avatar")?,
            platform: platform.clone(),
        };

        users.push(user);
    }

    Ok(users)
}
