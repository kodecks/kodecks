use crate::{app::AppState, token::Token, AppError};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::TypedHeader;
use headers::{authorization::Bearer, Authorization};
use k256::schnorr::signature::Verifier;
use kodecks::error::Error;
use kodecks_engine::login::{LoginRequest, LoginResponse, LoginType};
use std::sync::Arc;

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    state.check_client_version(&payload.client_version)?;
    match payload.ty {
        LoginType::PubkeyChallenge { pubkey } => {
            let challenge = state.new_session(&pubkey).challenge().to_string();
            Ok(Json(LoginResponse::Challenge { challenge }))
        }
        LoginType::PubkeyResponse { pubkey, signature } => {
            if let Some(session) = state.session_from_pubkey(&pubkey) {
                if pubkey
                    .verify(session.challenge().as_bytes(), &signature)
                    .is_ok()
                {
                    return Ok(Json(LoginResponse::Session {
                        token: session.token().to_string(),
                    }));
                }
            }
            Err(Error::FailedToConnectServer.into())
        }
    }
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> StatusCode {
    state.logout_by_token(&Token::from_str(authorization.token()));
    StatusCode::OK
}
