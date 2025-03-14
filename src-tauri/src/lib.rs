use std::collections::HashMap;

use anyhow::Result;
use axum::{routing::get, Router};
use emote::{Emote, EMOTES_CACHE};
use lazy_static::lazy_static;
use log::{error, info};
use tauri::{
    async_runtime::{self, Mutex},
    App, AppHandle, Manager,
};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_http::reqwest::{redirect::Policy, Client};
use tauri_plugin_store::StoreExt;
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
        .build()
        .unwrap();

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
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
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

            load_emotes(app);

            let app_handle = app.app_handle();

            async_runtime::block_on(async move {
                *APP_HANDLE.lock().await = Some(app_handle.clone());

                init_chat_state().await
            })?;

            async_runtime::spawn(async { start_api_server().await });

            Ok(())
        });

    builder
        .invoke_handler(tauri::generate_handler![
            user::fetch_live_now,
            user::fetch_full_user,
            user::fetch_stream_info,
        ])
        .run(tauri::generate_context!())
        .expect("while running tauri application");
}

fn load_emotes(app: &App) {
    let mut emotes: HashMap<String, HashMap<String, Emote>> = HashMap::new();

    let mut load = |store_name: &str| {
        info!("Loading emote store '{store_name}'");

        let store = match app.store(store_name) {
            Ok(store) => store,
            Err(err) => {
                error!("Failed to load emote store '{store_name}': {err}");
                return;
            }
        };

        for (username, stored_emotes) in store.entries() {
            if let Ok(stored_emotes) =
                serde_json::from_value::<HashMap<String, Emote>>(stored_emotes)
            {
                emotes.insert(username.to_string(), stored_emotes);
            } else {
                error!("Failed to deserialize '{store_name}' emotes for '{username}'");
            }
        }
    };

    load("user_emotes.json");
    load("seventv_emotes.json");
    load("bettertv_emotes.json");

    let mut cache = EMOTES_CACHE.lock().unwrap();
    *cache = emotes;

    info!("Loaded emotes for {} users", cache.len());
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
