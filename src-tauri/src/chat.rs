use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};

use crate::{
    emote::{Emote, EMOTES_CACHE},
    utils, CHAT_STATE,
};
use anyhow::Result;
use axum::{
    extract::Path,
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
};
use futures_util::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use serde::Serialize;
use tauri::async_runtime;
use tokio::{
    sync::{broadcast, mpsc},
    time,
};
use tokio_stream::wrappers::BroadcastStream;
use tokio_tungstenite::tungstenite::Message;

const WS_CHAT_URL: &str = "wss://irc-ws.chat.twitch.tv";
const PING: &str = "PING";
const PONG: &str = "PONG";

lazy_static! {
    static ref IRC_CHAT_REG: Regex = Regex::new(
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

#[derive(Serialize)]
pub struct ChatMessage {
    #[serde(rename = "c")]
    color: String,
    #[serde(rename = "n")]
    name: String,
    #[serde(rename = "f")]
    first_msg: bool,
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

pub async fn join_chat(
    Path(username): Path<String>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let user_emotes = {
        let user_emotes_lock = EMOTES_CACHE.lock().unwrap();
        if let Some(emotes) = user_emotes_lock.get(&username) {
            Arc::new(emotes.clone())
        } else {
            error!("Emotes not found for '{username}'");
            Arc::new(HashMap::default())
        }
    };

    let mut state = CHAT_STATE.lock().await;
    let state = state.as_mut().unwrap();

    let sender = state.sender.clone();

    if state.current_chat.is_some() {
        let old = state.current_chat.clone().unwrap();
        if old != username {
            info!("Leaving '{old}' chat");
            if let Err(err) = sender.send(format!("PART #{old}")).await {
                error!("Send: {err}");
            }

            state.current_chat = None;
        }
    }

    info!("Joining '{username}' chat");
    if sender.send(format!("JOIN #{username}")).await.is_ok() {
        state.current_chat = Some(username.to_string());
    } else {
        error!("Failed to join chat: {username}");
    }

    let rx = state.receiver.subscribe();

    let stream = BroadcastStream::new(rx).filter_map(move |result| {
        let user_emotes = user_emotes.clone();

        async move {
            match result {
                Ok(irc) => {
                    if let Some(caps) = IRC_CHAT_REG.captures(&irc) {
                        if caps.len() < 5 {
                            return None;
                        }
                        let color = caps.name("color")?.as_str().to_string();
                        let display_name = caps.name("display_name")?.as_str().to_string();
                        let first_msg = caps.name("first_msg")?.as_str() != "0";
                        let content = caps.name("message")?.as_str().trim_end();

                        if display_name.is_empty() || content.is_empty() {
                            return None;
                        }

                        let fragments = parse_chat_fragments(content, &user_emotes);
                        if fragments.is_empty() {
                            return None;
                        }

                        let chat_message = ChatMessage {
                            color,
                            name: display_name,
                            first_msg,
                            fragments,
                        };

                        let json = serde_json::to_string(&chat_message).ok()?;
                        Some(Ok(Event::default().data(json)))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
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
