use tauri::async_runtime;
use tauri_plugin_deep_link::DeepLinkExt;

mod api;
mod emotes;
mod proxy;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}));
    }

    builder = builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.deep_link().register("rt")?;

            async_runtime::spawn(async {
                if let Err(e) = api::start_api_server().await {
                    eprintln!("Failed to start axum server: {}", e);
                }
            });

            Ok(())
        });

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
