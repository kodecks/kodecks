use crate::app::AppState;
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(_payload): Json<Login>,
) -> (StatusCode, Json<Session>) {
    let user = Session {
        token: state.new_token(),
    };

    (StatusCode::CREATED, Json(user))
}

#[derive(Deserialize)]
pub struct Login {}

#[derive(Serialize)]
pub struct Session {
    token: String,
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> StatusCode {
    state.logout(authorization.token());
    StatusCode::OK
}
