use anyhow::anyhow;
use log::LevelFilter;
use sqlx::SqlitePool;
use tauri::{
    async_runtime::{self, Mutex},
    Manager,
};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_sql::{Migration, MigrationKind};

mod feeds;
mod twitch;
mod users;
mod utils;

pub struct AppState {
    pub users_db: Option<SqlitePool>,
    pub feeds_db: Option<SqlitePool>,
    pub emotes_db: Option<SqlitePool>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {
            /*
            let webview_url = WebviewUrl::App("index.html".into());

            if let Err(err) = WebviewWindowBuilder::new(app, "second", webview_url).build() {
                println!("Error creating new window: {err}");
            }
            */
        }));
    }

    builder = builder
        .plugin(
            tauri_plugin_sql::Builder::new()
                .add_migrations("sqlite:users.db", users_migrations())
                .add_migrations("sqlite:feeds.db", feeds_migrations())
                .add_migrations("sqlite:emotes.db", emotes_migrations())
                .build(),
        )
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(LevelFilter::Debug)
                .level_for("rustls", LevelFilter::Off)
                .level_for("h2", LevelFilter::Off)
                .level_for("hyper_util", LevelFilter::Off)
                .level_for("sqlx", LevelFilter::Info)
                .level_for(
                    "tao::platform_impl::platform::event_loop::runner",
                    LevelFilter::Off,
                )
                .build(),
        )
        .setup(|app| {
            #[cfg(desktop)]
            app.deep_link().register("rt")?;

            let app_data_path = app.path().app_data_dir()?;

            async_runtime::block_on(async {
                let users_db_path = app_data_path.join("users.db");
                let users_db = match SqlitePool::connect(users_db_path.to_str().unwrap()).await {
                    Ok(db) => db,
                    Err(err) => {
                        return Err(anyhow!("Failed to connect to users database: {err}"));
                    }
                };

                let feeds_db_path = app_data_path.join("feeds.db");
                let feeds_db = match SqlitePool::connect(feeds_db_path.to_str().unwrap()).await {
                    Ok(db) => db,
                    Err(err) => {
                        return Err(anyhow!("Failed to connect to feeds database: {err}"));
                    }
                };

                let emotes_db_path = app_data_path.join("emotes.db");
                let emotes_db = match SqlitePool::connect(emotes_db_path.to_str().unwrap()).await {
                    Ok(db) => db,
                    Err(err) => {
                        return Err(anyhow!("Failed to connect to emotes database: {err}"));
                    }
                };

                Ok(app.manage(Mutex::new(AppState {
                    users_db: Some(users_db),
                    feeds_db: Some(feeds_db),
                    emotes_db: Some(emotes_db),
                })))
            })?;

            Ok(())
        });

    builder
        .invoke_handler(tauri::generate_handler![
            users::get_users,
            users::add_user,
            users::remove_user,
            feeds::get_feed,
            feeds::refresh_feed,
            twitch::stream::fetch_stream_playback,
            twitch::proxy::proxy_stream,
            twitch::chat::join_chat,
        ])
        .run(tauri::generate_context!())
        .expect("while running tauri application");
}

fn users_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create_users_table",
        sql: r"
                CREATE TABLE IF NOT EXISTS twitch (
                    id TEXT,
                    username TEXT NOT NULL PRIMARY KEY,
                    avatar BLOB
                );
            ",
        kind: MigrationKind::Up,
    }]
}

fn feeds_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create_feeds_table",
        sql: r"
                CREATE TABLE IF NOT EXISTS twitch (
                    username TEXT NOT NULL PRIMARY KEY,
                    started_at TEXT
                );
            ",
        kind: MigrationKind::Up,
    }]
}

fn emotes_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create_emotes_table",
        sql: r"
                CREATE TABLE IF NOT EXISTS twitch (
                    username TEXT NOT NULL,
                    name TEXT NOT NULL,
                    url TEXT,
                    width INTEGER,
                    height INTEGER,
                    PRIMARY KEY (username, name)
                );
            ",
        kind: MigrationKind::Up,
    }]
}
