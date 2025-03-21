use lazy_static::lazy_static;
use log::LevelFilter;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_sql::{Migration, MigrationKind};
use tokio::sync::Mutex;
use twitch::user;

mod twitch;
mod utils;

#[derive(Default)]
pub struct AppState {
    pub emotes_db: Option<SqlitePool>,
}

lazy_static! {
    // Hold the AppHandle to allow events to be sent to the main window.
    pub static ref APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}));
    }

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
                .level(LevelFilter::Info)
                .level_for("reqwest", log::LevelFilter::Debug)
                .build(),
        )
        .setup(|app| {
            #[cfg(desktop)]
            app.deep_link().register("rt")?;

            let app_data_path = app.path().app_data_dir()?;

            twitch::main::setup(&app_data_path)?;

            Ok(())
        });

    builder
        .invoke_handler(tauri::generate_handler![
            user::fetch_live_now,
            user::fetch_full_user,
            user::fetch_stream_info,
            user::fetch_stream_playback,
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
