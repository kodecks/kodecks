use crate::app::AppState;
use axum::{
    extract::{ws::WebSocket, ConnectInfo, Query, State, WebSocketUpgrade},
    response::IntoResponse,
};
use http::StatusCode;
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};
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
        ws.on_upgrade(move |socket| handle_socket(socket, addr))
            .into_response()
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if let Err(err) = socket.send(msg).await {
                error!("client {who} disconnected: {err}");
                return;
            }
        } else {
            error!("client {who} abruptly disconnected");
            return;
        }
    }
    println!("client {who} disconnected");
}
