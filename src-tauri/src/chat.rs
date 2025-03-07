use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{
    api::AppState,
    emote::{Emote, EMOTES_CACHE},
    utils,
};
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
};
use futures_util::{stream::BoxStream, SinkExt, StreamExt};
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use serde::Serialize;
use tauri::async_runtime::{self, Mutex};
use tokio::{
    sync::{broadcast, mpsc},
    time,
};
use tokio_tungstenite::tungstenite::Message;

const WS_CHAT_URL: &str = "wss://irc-ws.chat.twitch.tv";

#[derive(Serialize)]
struct ChatMessage {
    #[serde(rename = "c")]
    color: String,
    #[serde(rename = "n")]
    name: String,
    #[serde(rename = "f")]
    first_msg: String,
    #[serde(rename = "m")]
    fragments: Vec<Fragment>,
}

#[derive(Serialize)]
struct Fragment {
    #[serde(rename = "t")]
    r#type: u8,
    #[serde(rename = "c")]
    content: String,
    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    emote: Option<Emote>,
}

lazy_static! {
    pub static ref CURRENT_CHAT: Mutex<Option<String>> = Mutex::new(None);

    static ref IRC_CHAT_REGEX: Regex = Regex::new(
         r"(?m)^@.*?color=(?P<color>[^;]*).*?display-name=(?P<display_name>[^;]*).*?first-msg=(?P<first_msg>[^;]*).*?PRIVMSG\s+\S+\s+:(?P<message>.*)$"
    ).unwrap();

    static ref URL_REG: Regex = Regex::new(
        r"(?m)(https?:\/\/)?(www\.)?([a-zA-Z0-9-]{1,256})\.[a-zA-Z0-9]{2,}(\/[^\s]*)?"
    ).unwrap();
}

pub async fn init_irc_connection() -> Result<(mpsc::Sender<String>, broadcast::Sender<String>)> {
    let (mut ws_stream, _) = tokio_tungstenite::connect_async(WS_CHAT_URL).await?;

    ws_stream.send("CAP REQ :twitch.tv/tags".into()).await?;
    ws_stream.send("PASS SCHMOOPIIE".into()).await?;

    let random_number = utils::random_number(10_000, 99_999);
    ws_stream
        .send(format!("NICK justinfan{random_number}").into())
        .await?;

    let (ws_sender_tx, mut ws_sender_rx) = mpsc::channel::<String>(100);
    let (ws_broadcast_tx, _) = broadcast::channel::<String>(100);

    let state = Arc::new(AppState {
        ws_sender: ws_sender_tx.clone(),
        ws_broadcast: ws_broadcast_tx.clone(),
    });

    async_runtime::spawn({
        let state = state.clone();
        async move {
            let (mut ws_sink, mut ws_stream) = ws_stream.split();
            let broadcast = state.ws_broadcast.clone();

            let ping = String::from("PING");
            let pong = String::from("PONG");

            loop {
                let ping_clone = ping.clone();

                tokio::select! {
                    maybe_msg = ws_stream.next() => {
                        match maybe_msg {
                            Some(Ok(msg)) => {
                                if let Message::Text(utf8) = msg {
                                    let text = utf8.to_string();

                                    if text.starts_with("PING") {
                                        if let Err(err) = ws_sink.send(Message::text(&pong)).await {
                                            error!("Failed to respond to ping: {err}");
                                        } else {
                                            let sender_clone = state.ws_sender.clone();

                                            tokio::spawn(async move {
                                                time::sleep(Duration::from_secs(60)).await;
                                                if let Err(err) = sender_clone.send(ping_clone).await {
                                                    error!("Failed to send scheduled ping: {err}");
                                                }
                                            });
                                        }
                                    }  else {
                                        let _ = broadcast.send(text);
                                    }
                                }
                            },
                            Some(Err(err)) => {
                                if err.to_string().is_empty() {
                                    continue;
                                }

                                error!("WebSocket error: {err}");
                                break;
                            }
                            None => break,
                        }
                    },
                    maybe_sender_msg = ws_sender_rx.recv() => {
                        if let Some(msg) = maybe_sender_msg {
                            if let Err(err) = ws_sink.send(msg.into()).await {
                                error!("Failed to send message to WebSocket: {err}");
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    });

    Ok((ws_sender_tx, ws_broadcast_tx))
}

pub async fn join_chat(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    if username.is_empty() {
        error!("Empty username");
        return StatusCode::BAD_REQUEST.into_response();
    }

    let user_emotes = {
        let user_emotes_lock = EMOTES_CACHE.lock().await;
        if let Some(emotes) = user_emotes_lock.get(&username) {
            Arc::new(emotes.clone())
        } else {
            error!("Emotes not found for '{username}'");
            Arc::new(HashMap::default())
        }
    };

    let mut current_stream = CURRENT_CHAT.lock().await;

    if current_stream.is_some() {
        let old = current_stream.clone().unwrap();
        if old != username {
            info!("Leaving '{old}' chat");
            if let Err(err) = state.ws_sender.send(format!("PART #{old}")).await {
                error!("Send: {err}");
            }

            *current_stream = None;
        }
    }

    info!("Joining '{username}' chat");
    if state
        .ws_sender
        .send(format!("JOIN #{username}"))
        .await
        .is_ok()
    {
        *current_stream = Some(username.to_string());
    } else {
        error!("Failed to join chat: {username}");
    }

    let mut rx = state.ws_broadcast.subscribe();

    let events_stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(text) => {
                    let caps = IRC_CHAT_REGEX.captures(&text);
                            if caps.is_none() {
                                continue;
                            }

                            let Some(caps) = caps else { continue };

                            if caps.len() < 5 {
                                continue;
                            }

                            let color = caps.name("color").unwrap().as_str().to_string();
                            let name = caps.name("display_name").unwrap().as_str().to_string();
                            let first_msg = caps.name("first_msg").unwrap().as_str().to_string();

                            let text = caps
                                .name("message")
                                .unwrap()
                                .as_str()
                                .trim_end();

                            let fragments =  parse_chat_fragments(text, &user_emotes);

                            let chat_message = ChatMessage {
                                color,
                                name,
                                first_msg,
                                fragments,
                            };

                            let mut json = match serde_json::to_string(&chat_message) {
                                Ok(json) => json,
                                Err(err) => {
                                    error!("Failed to serialize chat message: {err}");
                                    continue;
                                }
                            };

                            json.push('\n');
                            yield Ok(Event::default().data(json));
                },
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    let events: BoxStream<'static, Result<Event, axum::Error>> = events_stream.boxed();

    Sse::new(events)
        .keep_alive(KeepAlive::default())
        .into_response()
}

fn parse_chat_fragments(
    message_content: &str,
    user_emotes: &HashMap<String, Emote>,
) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    // This initializer value was revealed to me in a dream
    let mut last_type = 10;

    for token in message_content.split_whitespace() {
        let current_type;

        if URL_REG.is_match(token) {
            current_type = 2;
        } else if user_emotes.contains_key(token) {
            current_type = 1;
        } else {
            current_type = 0;
        }

        if current_type != last_type {
            let emote = if current_type == 1 {
                user_emotes.get(token).cloned()
            } else {
                None
            };

            fragments.push(Fragment {
                r#type: current_type,
                content: token.to_string(),
                emote,
            });

            last_type = current_type;
            continue;
        }

        if current_type == 0 {
            // Append to last fragment with an whitespace
            fragments
                .last_mut()
                .unwrap()
                .content
                .push_str(format!(" {token}").as_str());
        }
    }

    fragments
}
