use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use tauri::{
    async_runtime::{self, Mutex},
    AppHandle, Theme, WebviewUrl, WebviewWindowBuilder,
};

lazy_static! {
    static ref WINDOW_ID: Mutex<u64> = Mutex::new(0);
    static ref TWITCH_URL_REG: Regex = Regex::new(r"twitch.tv/([a-zA-Z0-9_]+)").unwrap();
}

pub fn open_url(app_handle: AppHandle, urls: &[String]) -> Result<()> {
    let mut username = String::new();

    if urls.is_empty() {
        return Err(anyhow!("No URLs provided"));
    }

    let url = urls.first().unwrap();

    if let Some(caps) = TWITCH_URL_REG.captures(url) {
        if let Some(m) = caps.get(1) {
            username = m.as_str().to_string();
        }
    } else {
        let cleaned = url.trim_start_matches("rt:/").trim();
        if let Some(first_part) = cleaned.split('/').next() {
            username = first_part.to_string();
        }
    }

    if username.is_empty() {
        return Err(anyhow!("Username was empty after parsing URL"));
    }

    let url = format!("/watch/twitch?username={username}");

    open_new_window(app_handle, url);

    Ok(())
}

#[tauri::command]
pub fn open_new_window(app_handle: AppHandle, url: String) {
    // In Windows, a new window must be created in a separate thread
    async_runtime::spawn(async move {
        let webview_url = WebviewUrl::App(url.into());

        let mut window_id = WINDOW_ID.lock().await;
        *window_id += 1;

        if let Err(err) =
            WebviewWindowBuilder::new(&app_handle, format!("rt-{window_id}"), webview_url)
                .theme(Some(Theme::Dark))
                .shadow(false)
                .decorations(false)
                .build()
        {
            println!("Error creating new window: {err}");
        }
    });
}
