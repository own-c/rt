use anyhow::anyhow;
use log::error;
use tauri::{
    async_runtime::{self, Mutex},
    Manager,
};
use tauri_plugin_deep_link::DeepLinkExt;
use tokio::sync::{broadcast, mpsc};

mod api;
mod chat;
mod emote;
mod proxy;
mod user;
mod utils;

pub struct AppState {
    current_stream: Option<String>,
    /// For sending messages to the chat websocket.
    pub sender: mpsc::Sender<String>,
    /// For receiving messages from the websocket.
    pub receiver: broadcast::Sender<String>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    async_runtime::spawn(async {
        if let Err(err) = api::start_api_server().await {
            error!("Failed to start axum server: {err}");
        }
    });

    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}));
    }

    builder = builder
        .plugin(
            tauri_plugin_log::Builder::new()
                .level_for("reqwest", log::LevelFilter::Debug)
                .level_for("rustls", log::LevelFilter::Off)
                .level_for("tungstenite", log::LevelFilter::Off)
                .level_for("tokio_tungstenite", log::LevelFilter::Off)
                .build(),
        )
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.deep_link().register("rt")?;

            async_runtime::block_on(async move {
                let (ws_sender, ws_receiver) = match chat::init_irc_connection().await {
                    Ok(channels) => channels,
                    Err(err) => {
                        return Err(anyhow!("Failed to initialize WebSocket connection: {err}"));
                    }
                };

                app.manage(Mutex::new(AppState {
                    current_stream: None,
                    sender: ws_sender,
                    receiver: ws_receiver,
                }));

                Ok(())
            })?;

            Ok(())
        });

    builder
        .invoke_handler(tauri::generate_handler![
            user::get_user,
            user::get_live_now,
            chat::join_chat,
            chat::leave_chat
        ])
        .run(tauri::generate_context!())
        .expect("while running tauri application");
}
