use std::time::Duration;

use anyhow::Result;
use axum::{routing::get, Router};
use lazy_static::lazy_static;
use log::{info, LevelFilter};
use sqlx::SqlitePool;
use tauri::{
    async_runtime::{self, Mutex},
    AppHandle, Manager,
};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_http::reqwest::{redirect::Policy, Client};
use tauri_plugin_sql::{Migration, MigrationKind};
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc},
};
use tower_http::cors::{Any, CorsLayer};
mod chat;
mod emote;
mod proxy;
mod queries;
mod user;
mod utils;

pub const LOCAL_API_ADDR: &str = "127.0.0.1:3030";

pub const GRAPHQL_API: &str = "https://gql.twitch.tv/gql";

pub struct ChatState {
    current_chat: Option<String>,
    /// For sending messages to the chat websocket.
    sender: mpsc::Sender<String>,
    /// For receiving messages from the websocket.
    receiver: broadcast::Sender<String>,
}

#[derive(Default)]
pub struct AppState {
    pub emotes_db: Option<SqlitePool>,
}

lazy_static! {
    pub static ref CHAT_STATE: Mutex<Option<ChatState>> = Mutex::new(None);

    // Used for various requests.
    pub static ref HTTP_CLIENT: Client = Client::builder()
        .gzip(true)
        .use_rustls_tls()
        .redirect(Policy::none())
        .https_only(true)
        .http2_prior_knowledge()
        .build()
        .unwrap();

    // Specifically used in the proxy.
    pub static ref PROXY_HTTP_CLIENT: Client = Client::builder()
        .gzip(true)
        .use_rustls_tls()
        .https_only(true)
        .tcp_keepalive(Duration::from_secs(5))
        .build()
        .unwrap();

    // Hold the AppHandle to allow events to be sent to the main window.
    pub static ref APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);

    pub static ref APP_STATE: Mutex<AppState> = Mutex::new(AppState::default());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    async_runtime::spawn(async { start_api_server().await });

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
                .level_for("rustls", log::LevelFilter::Off)
                .level_for("tungstenite", log::LevelFilter::Off)
                .level_for("tokio_tungstenite", log::LevelFilter::Off)
                .level_for(
                    "tao::platform_impl::platform::event_loop::runner",
                    log::LevelFilter::Off,
                )
                .build(),
        )
        .setup(|app| {
            #[cfg(desktop)]
            app.deep_link().register("rt")?;

            let handle = app.app_handle();
            let path = app.path().app_data_dir()?;

            async_runtime::block_on(async move {
                let db_path = path.join("emotes.db");
                let db = SqlitePool::connect(db_path.to_str().unwrap()).await?;

                *APP_STATE.lock().await = AppState {
                    emotes_db: Some(db),
                };

                *APP_HANDLE.lock().await = Some(handle.clone());

                init_chat_state().await
            })?;

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

async fn init_chat_state() -> Result<()> {
    let (ws_sender, ws_receiver) = chat::init_irc_connection().await?;

    let chat = Some(ChatState {
        current_chat: None,
        sender: ws_sender,
        receiver: ws_receiver,
    });

    let mut state = CHAT_STATE.lock().await;
    *state = chat;

    info!("Initialized chat state");
    Ok(())
}

async fn start_api_server() -> Result<()> {
    let cors_layer = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    info!("Binding API server on {LOCAL_API_ADDR}");
    let listener = TcpListener::bind(LOCAL_API_ADDR).await?;

    let app = Router::new()
        .route("/chat/{username}", get(chat::join_chat))
        .route("/proxy", get(proxy::proxy_stream))
        .layer(cors_layer);

    info!("Starting API server");
    axum::serve(listener, app).await?;
    Ok(())
}
