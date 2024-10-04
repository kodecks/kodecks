use crate::app::AppState;
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use k256::schnorr::signature::Verifier;
use kodecks_engine::login::{LoginRequest, LoginResponse, LoginType};
use serde::Serialize;
use std::sync::Arc;

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<LoginResponse>) {
    match payload.ty {
        LoginType::PubkeyChallenge { pubkey } => {
            let challenge = state.new_session(&pubkey).challenge().to_string();
            (StatusCode::OK, Json(LoginResponse::Challenge { challenge }))
        }
        LoginType::PubkeyResponse { pubkey, signature } => {
            if let Some(session) = state.session_from_pubkey(&pubkey) {
                if pubkey
                    .verify(session.challenge().as_bytes(), &signature)
                    .is_ok()
                {
                    return (
                        StatusCode::OK,
                        Json(LoginResponse::Session {
                            token: session.token().to_string(),
                        }),
                    );
                }
            }
            (StatusCode::UNAUTHORIZED, Json(LoginResponse::Failed))
        }
    }
}

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
