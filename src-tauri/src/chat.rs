use std::{convert::Infallible, sync::Arc};

use anyhow::Result;
use async_runtime::Mutex;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Response, Sse,
    },
};
use futures_util::{stream, SinkExt, StreamExt};
use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use tauri::async_runtime;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::utils;

const WS_CHAT_URL: &str = "wss://irc-ws.chat.twitch.tv";

lazy_static! {
    pub static ref CURRENT_CHAT: Mutex<Option<String>> = Mutex::new(None);

    static ref CHAT_REGEX: Regex = Regex::new(
        r"(?m)^@.*?color=(?P<color>[^;]*).*?display-name=(?P<display_name>[^;]*).*?first-msg=(?P<first_msg>[^;]*).*?tmi-sent-ts=(?P<tmi_sent_ts>[^;]*).*?PRIVMSG\s+\S+\s+:(?P<message>.*)$"
    ).unwrap();
}

pub async fn init_irc_connection() -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    info!("Connecting to IRC server {}", WS_CHAT_URL);
    let (mut ws_stream, _) = tokio_tungstenite::connect_async(WS_CHAT_URL).await?;

    ws_stream.send("CAP REQ :twitch.tv/tags".into()).await?;
    ws_stream.send("PASS SCHMOOPIIE".into()).await?;

    let random_number = utils::random_number(10_000, 99_999);
    ws_stream
        .send(format!("NICK justinfan{random_number}").into())
        .await?;

    ws_stream.flush().await?;

    Ok(ws_stream)
}

pub async fn join_chat(
    //                                       I love rust
    State(ws_stream): State<Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    username: Path<String>,
) -> impl IntoResponse {
    let Path(username) = username;

    if username.is_empty() {
        error!("No username provided");
        return (StatusCode::BAD_REQUEST, Response::default());
    }

    let ws_stream = ws_stream.clone();
    let (tx, rx) = async_runtime::channel::<Result<Event, Infallible>>(100);

    async_runtime::spawn(async move {
        let mut current_stream = CURRENT_CHAT.lock().await;
        let mut ws = ws_stream.lock().await;

        if current_stream.is_some() {
            let old = current_stream.clone().unwrap();
            if old != username {
                info!("Leaving '{old}' chat");
                if let Err(err) = ws.send(format!("PART #{old}").into()).await {
                    error!("Send: {err}");
                }

                *current_stream = None;
            }
        }

        info!("Joining '{username}' chat");
        if ws.send(format!("JOIN #{username}").into()).await.is_ok() {
            *current_stream = Some(username.to_string());
        } else {
            error!("Failed to join chat: {username}");
        }

        loop {
            match ws.next().await {
                Some(Ok(msg)) => {
                    if let Message::Text(text) = msg {
                        let caps = CHAT_REGEX.captures(&text);
                        if caps.is_none() {
                            continue;
                        }

                        let Some(caps) = caps else { continue };

                        if caps.len() < 5 {
                            error!("Not enough captures for '{}'", text);
                            continue;
                        }

                        let color = caps.name("color").unwrap().as_str().to_string();
                        let name = caps.name("display_name").unwrap().as_str().to_string();
                        let first_msg = caps.name("first_msg").unwrap().as_str().to_string();
                        let timestamp = caps.name("tmi_sent_ts").unwrap().as_str().to_string();
                        let message = caps
                            .name("message")
                            .unwrap()
                            .as_str()
                            .trim_end()
                            .to_string();

                        let chat_message = format!(
                            "$TIMESTAMP: {timestamp} $COLOR: {color} $FIRST_MSG: {first_msg} $NAME: {name} $MESSAGE: {message}"
                        );

                        match tx.send(Ok(Event::default().data(&chat_message))).await {
                            Ok(_) => {}
                            Err(err) => {
                                error!("Error sending chat message: {err}");
                                break;
                            }
                        }
                    }
                }
                Some(Err(err)) => {
                    if err.to_string().is_empty() {
                        error!("WebSocket error: empty error");
                        return;
                    }

                    error!("WebSocket error: {err:?}");
                    break;
                }
                None => break,
            }
        }
    });

    let event_stream = stream::unfold(rx, |mut rx| async {
        rx.recv().await.map(|event| (event, rx))
    });

    (
        StatusCode::OK,
        Sse::new(event_stream)
            .keep_alive(KeepAlive::default())
            .into_response(),
    )
}
