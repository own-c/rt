use log::error;
use tauri::async_runtime;
use tauri_plugin_deep_link::DeepLinkExt;
use user::User;

mod api;
mod chat;
mod emote;
mod proxy;
mod user;
mod utils;

#[tauri::command]
async fn get_live_now(usernames: Vec<String>) -> Result<Vec<String>, String> {
    match user::get_live_now(usernames).await {
        Ok(live) => Ok(live),
        Err(err) => {
            error!("get_live_now failed: {err}");
            Err(err.to_string())
        }
    }
}

#[tauri::command]
async fn get_user(username: String) -> Result<User, String> {
    match user::get_user(username).await {
        Ok(stream) => Ok(stream),
        Err(err) => {
            error!("get_user_stream failed: {err}");
            Err(err.to_string())
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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

            async_runtime::spawn(async {
                if let Err(err) = api::start_api_server().await {
                    error!("Failed to start axum server: {err}");
                }
            });

            Ok(())
        });

    builder
        .invoke_handler(tauri::generate_handler![get_user, get_live_now,])
        .run(tauri::generate_context!())
        .expect("while running tauri application");
}
