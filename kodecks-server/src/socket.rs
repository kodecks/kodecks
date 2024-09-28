use crate::{app::AppState, game::PlayerData};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use http::StatusCode;
use kodecks_engine::message::{Command, Input};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::{select, sync::broadcast, sync::mpsc};
use tracing::*;

#[derive(Deserialize)]
pub struct SocketAuth {
    token: String,
}

pub async fn ws_handler(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
    auth: Query<SocketAuth>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    if let Some(mut session) = state.session_mut(&auth.token) {
        session.update();
        std::mem::drop(session);
        let state = state.clone();
        ws.on_upgrade(move |socket| handle_socket(state, socket, addr))
            .into_response()
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

async fn handle_socket(state: Arc<AppState>, mut socket: WebSocket, who: SocketAddr) {
    let (command_sender, command_receiver) = broadcast::channel(256);
    let (event_sender, mut event_receiver) = mpsc::channel(256);

    loop {
        select! {
            Some(event) = event_receiver.recv() => {
                let event = match serde_json::to_string(&event) {
                    Ok(event) => event,
                    Err(err) => {
                        warn!("failed to serialize event: {}", err);
                        continue;
                    }
                };
                if let Err(err) = socket.send(Message::Text(event)).await {
                    warn!("failed to send event: {}", err);
                    break;
                }
            }
            msg = socket.recv() => {
                if let Some(Ok(Message::Text(msg))) = msg {
                    let command = match serde_json::from_str(&msg) {
                        Ok(msg) => msg,
                        Err(err) => {
                            warn!("failed to parse message: {}", err);
                            break;
                        }
                    };
                    if let Input::Command(Command::StartRandomMatch { deck }) = &command {
                        let player = PlayerData {
                            deck: deck.clone(),
                            command_receiver: command_receiver.resubscribe(),
                            event_sender: event_sender.clone(),
                        };
                        state.add_to_random_match_pool(player);
                    } else if let Err(err) = command_sender.send(command) {
                        warn!("failed to send command: {}", err);
                        break;
                    }
                } else {
                    break;
                }
            }
            else => break,
        }
    }

    info!("client {who} disconnected");
}
