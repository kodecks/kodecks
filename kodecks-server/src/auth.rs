use crate::app::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use std::sync::Arc;

pub async fn auth(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(mut session) = state.session_mut(authorization.token()) {
        session.update();
        std::mem::drop(session);
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
