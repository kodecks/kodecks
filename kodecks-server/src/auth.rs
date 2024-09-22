use crate::app::AppState;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub async fn auth(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(token) = get_token(&headers) {
        if let Some(mut session) = state.session_mut(token) {
            session.update();
            let response = next.run(request).await;
            return Ok(response);
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

pub fn get_token(headers: &HeaderMap) -> Option<&str> {
    headers.get("Authorization").and_then(|value| {
        value.to_str().ok().and_then(|value| {
            let mut parts = value.splitn(2, ' ');
            if parts.next()? == "Bearer" {
                parts.next()
            } else {
                None
            }
        })
    })
}
