use crate::{app::AppState, token::Token};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use http::StatusCode;
use kodecks_engine::{message::Input, user::UserId};
use serde::Deserialize;
use std::sync::Arc;
use tokio::{select, sync::mpsc};
use tracing::*;

#[derive(Deserialize)]
pub struct SocketAuth {
    token: Token,
}

pub async fn ws_handler(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
    auth: Query<SocketAuth>,
) -> impl IntoResponse {
    if let Some(mut session) = state.session_from_token_mut(&auth.token) {
        session.update();
        let id = session.user_id().clone();
        std::mem::drop(session);
        let state = state.clone();
        ws.on_upgrade(move |socket| handle_socket(state, socket, id))
            .into_response()
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

async fn handle_socket(state: Arc<AppState>, mut socket: WebSocket, user_id: UserId) {
    let (event_sender, mut event_receiver) = mpsc::channel(256);
    let config = bincode::config::standard();

    if let Some(mut session) = state.session_from_id_mut(&user_id) {
        session.set_event_sender(Some(event_sender.clone()));
    }

    loop {
        if let Some(mut session) = state.session_from_id_mut(&user_id) {
            session.update();
        }
        select! {
            Some(event) = event_receiver.recv() => {
                let event = match bincode::encode_to_vec(&event, config) {
                    Ok(event) => event,
                    Err(err) => {
                        warn!("failed to serialize event: {}", err);
                        continue;
                    }
                };
                if let Err(err) = socket.send(Message::Binary(event)).await {
                    warn!("failed to send event: {}", err);
                    break;
                }
            }
            msg = socket.recv() => {
                if let Some(Ok(Message::Binary(msg))) = msg {
                    let command: Input = match bincode::decode_from_slice(&msg, config) {
                        Ok((msg, _)) => msg,
                        Err(err) => {
                            warn!("failed to parse message: {}", err);
                            break;
                        }
                    };

                    state.handle_command(&user_id, command.clone());
                } else {
                    break;
                }
            }
            else => break,
        }
    }

    state.logout(&user_id);
    info!("client {user_id} disconnected");
}
