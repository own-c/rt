use lazy_static::lazy_static;
use log::LevelFilter;
use tauri::{
    async_runtime::{self, Mutex},
    AppHandle, Manager,
};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_sql::{Migration, MigrationKind};

use twitch::{chat, proxy, user};

mod twitch;
mod utils;

lazy_static! {
    // Hold the AppHandle to allow events to be sent to the main window.
    pub static ref APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    builder = builder
        .plugin(
            tauri_plugin_sql::Builder::new()
                .add_migrations("sqlite:emotes.db", emotes_migrations())
                .build(),
        )
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(LevelFilter::Debug)
                .level_for("rustls", LevelFilter::Off)
                .build(),
        )
        .setup(|app| {
            #[cfg(desktop)]
            app.deep_link().register("rt")?;

            let app_data_path = app.path().app_data_dir()?;

            async_runtime::block_on(async {
                *APP_HANDLE.lock().await = Some(app.handle().clone());
            });

            twitch::main::setup(&app_data_path)?;

            Ok(())
        });

    builder
        .invoke_handler(tauri::generate_handler![
            user::fetch_live_now,
            user::fetch_full_user,
            user::fetch_stream_info,
            user::fetch_stream_playback,
            chat::join_chat,
            proxy::proxy_stream,
        ])
        .run(tauri::generate_context!())
        .expect("while running tauri application");
}

fn emotes_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "create_emotes_table",
        sql: r"
                CREATE TABLE IF NOT EXISTS emotes (
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
