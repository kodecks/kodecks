use crate::{app::AppState, game::PlayerData};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use http::StatusCode;
use kodecks_engine::message::{Command, Input};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::mpsc;
use tracing::error;

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

async fn handle_socket(state: Arc<AppState>, socket: WebSocket, who: SocketAddr) {
    let (command_sender, command_receiver) = mpsc::channel(256);
    let (event_sender, mut event_receiver) = mpsc::channel(256);

    let mut player = Some(PlayerData {
        deck: Default::default(),
        command_receiver,
        event_sender,
    });

    let (mut sender, mut receiver) = socket.split();

    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            if let Ok(event) = serde_json::to_string(&event) {
                sender.send(Message::Text(event)).await.unwrap();
            }
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(Message::Text(msg)) = msg {
                if let Ok(command) = serde_json::from_str(&msg) {
                    if let Input::Command(Command::StartRandomMatch { deck }) = &command {
                        if let Some(player) = player.take() {
                            state.add_to_random_match_pool(PlayerData {
                                deck: deck.clone(),
                                ..player
                            });
                        }
                    } else {
                        command_sender.try_send(command).unwrap();
                    }
                }
            } else {
                error!("client {who} abruptly disconnected");
                break;
            }
        }
    });

    println!("client {who} disconnected");
}
