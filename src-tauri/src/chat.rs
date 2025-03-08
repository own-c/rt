use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{
    emote::{Emote, EMOTES_CACHE},
    utils,
};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use serde::Serialize;
use tauri::{
    async_runtime::{self, Mutex},
    ipc::Channel,
};
use tokio::{
    sync::{broadcast, mpsc},
    time,
};
use tokio_tungstenite::tungstenite::Message;

const WS_CHAT_URL: &str = "wss://irc-ws.chat.twitch.tv";
const PING: &str = "PING";
const PONG: &str = "PONG";

lazy_static! {
    static ref CURRENT_CHAT_NAME: Mutex<Option<String>> = Mutex::new(None);

    static ref IRC_CHAT_REG: Regex = Regex::new(
         r"(?m)^@.*?color=(?P<color>[^;]*).*?display-name=(?P<display_name>[^;]*).*?first-msg=(?P<first_msg>[^;]*).*?PRIVMSG\s+\S+\s+:(?P<message>.*)$"
    ).unwrap();

    static ref URL_REG: Regex = Regex::new(
        r"(?m)(https?:\/\/)?(www\.)?([a-zA-Z0-9-]{1,256})\.[a-zA-Z0-9]{2,}(\/[^\s]*)?"
    ).unwrap();

     static ref WEBSOCKET_CHANNELS: Mutex<Option<WebSocket>> = Mutex::new(None);
}

pub struct WebSocket {
    /// For sending messages to the websocket.
    pub sender: Option<mpsc::Sender<String>>,
    /// For sending received websocket messages to the Tauri channel.
    pub broadcast: Option<broadcast::Sender<String>>,
}

pub async fn init_irc_connection() -> Result<()> {
    let (mut ws_stream, _) = tokio_tungstenite::connect_async(WS_CHAT_URL).await?;

    ws_stream.send("CAP REQ :twitch.tv/tags".into()).await?;
    ws_stream.send("PASS SCHMOOPIIE".into()).await?;

    let random_number = utils::random_number(10_000, 99_999);

    ws_stream
        .send(format!("NICK justinfan{random_number}").into())
        .await?;

    let (ws_sender_tx, mut ws_sender_rx) = mpsc::channel::<String>(100);
    let (ws_broadcast_tx, _) = broadcast::channel::<String>(100);

    async_runtime::spawn({
        let sender = ws_sender_tx.clone();
        let broadcast = ws_broadcast_tx.clone();

        async move {
            let (mut ws_sink, mut ws_stream) = ws_stream.split();

            loop {
                tokio::select! {
                    // Process incoming WebSocket messages.
                    msg_result = ws_stream.next() => {
                        match msg_result {
                            Some(Ok(Message::Text(text))) => {
                                if text.starts_with(PING) {
                                    if let Err(err) = ws_sink.send(Message::text(PONG)).await {
                                        error!("Failed to send PONG: {err}");
                                    } else {
                                        let sender_clone = sender.clone();

                                        // Ping the server after 60 seconds.
                                        async_runtime::spawn(async move {
                                            time::sleep(Duration::from_secs(60)).await;

                                            if let Err(err) = sender_clone.send(PING.into()).await {
                                                error!("Failed to send scheduled PING: {err}");
                                            }
                                        });
                                    }
                                } else {
                                    // Broadcast non-ping messages.
                                    let _ = broadcast.send(text.to_string());
                                }
                            },
                            // Optionally handle other message types or ignore them.
                            Some(Ok(_)) => {},
                            Some(Err(err)) => {
                                if !err.to_string().is_empty() {
                                    error!("WebSocket error: {err}");
                                    break;
                                }
                            },
                            None => break,
                        }
                    },
                    // Process messages coming from sender.
                    sender_msg = ws_sender_rx.recv() => {
                        if let Some(msg) = sender_msg {
                            if let Err(err) = ws_sink.send(msg.into()).await {
                                error!("Failed to send message: {err}");
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

    let mut lock = WEBSOCKET_CHANNELS.lock().await;
    *lock = Some(WebSocket {
        sender: Some(ws_sender_tx),
        broadcast: Some(ws_broadcast_tx),
    });

    Ok(())
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum ChatEvent {
    #[serde(rename_all = "camelCase")]
    Message(ChatMessage),
}

#[derive(Serialize, Clone)]
pub struct ChatMessage {
    #[serde(rename = "c")]
    color: String,
    #[serde(rename = "n")]
    name: String,
    #[serde(rename = "f")]
    first_msg: String,
    #[serde(rename = "m")]
    fragments: Vec<Fragment>,
}

#[derive(Serialize, Clone)]
struct Fragment {
    #[serde(rename = "t")]
    r#type: u8,
    #[serde(rename = "c")]
    content: String,
    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    emote: Option<Emote>,
}

#[tauri::command]
pub async fn join_chat(username: String, on_event: Channel<ChatEvent>) {
    let user_emotes = {
        let user_emotes_lock = EMOTES_CACHE.lock().await;
        if let Some(emotes) = user_emotes_lock.get(&username) {
            Arc::new(emotes.clone())
        } else {
            error!("Emotes not found for '{username}'");
            Arc::new(HashMap::default())
        }
    };

    let lock = WEBSOCKET_CHANNELS.lock().await;
    let ws = lock.as_ref().unwrap();
    let sender = ws.sender.clone().unwrap();
    let broadcast = ws.broadcast.clone().unwrap();
    drop(lock);

    let mut current_stream = CURRENT_CHAT_NAME.lock().await;

    if current_stream.is_some() {
        let old = current_stream.clone().unwrap();
        if old != username {
            info!("Leaving '{old}' chat");
            if let Err(err) = sender.send(format!("PART #{old}")).await {
                error!("Send: {err}");
            }

            *current_stream = None;
        }
    }

    info!("Joining '{username}' chat");
    if sender.send(format!("JOIN #{username}")).await.is_ok() {
        *current_stream = Some(username.to_string());
    } else {
        error!("Failed to join chat: {username}");
    }

    drop(current_stream);

    let mut rx = broadcast.subscribe();

    while let Ok(irc) = rx.recv().await {
        // From here its possible to parse all events coming from the chat, but for now we're only interested in messages.
        let caps = IRC_CHAT_REG.captures(&irc);
        if caps.is_none() {
            continue;
        }

        let Some(caps) = caps else { continue };

        if caps.len() < 5 {
            continue;
        }

        let color = caps.name("color").unwrap().as_str().to_string();
        let display_name = caps.name("display_name").unwrap().as_str().to_string();
        let first_msg = caps.name("first_msg").unwrap().as_str().to_string();
        let content = caps.name("message").unwrap().as_str().trim_end();

        if display_name.is_empty() || content.is_empty() {
            continue;
        }

        let fragments = parse_chat_fragments(content, &user_emotes);
        if fragments.is_empty() {
            continue;
        }

        let chat_message = ChatMessage {
            color,
            name: display_name,
            first_msg,
            fragments,
        };

        if let Err(err) = on_event.send(ChatEvent::Message(chat_message)) {
            error!("Failed to send chat event: {err}");
        }
    }
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
