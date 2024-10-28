use axum::{
    middleware,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Method, StatusCode,
};
use kodecks::error::Error;
use std::{net::SocketAddr, sync::Arc};
use tokio::try_join;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod app;
mod auth;
mod background;
mod game;
mod login;
mod room;
mod session;
mod socket;
mod token;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(3000);
    let addr = ("0.0.0.0", port);
    info!("Listening on {}:{}", addr.0, addr.1);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_origin(Any);

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(8)
            .finish()
            .unwrap(),
    );

    let state = Arc::new(app::AppState::new());

    let authorized = Router::new()
        .route("/logout", get(login::logout))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::auth));

    let app = Router::new()
        .route("/", get(|| async { Redirect::temporary("/status") }))
        .route("/status", get(app::status))
        .route("/login", post(login::login))
        .route("/ws", get(socket::ws_handler))
        .layer(GovernorLayer {
            config: governor_conf,
        })
        .merge(authorized)
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(state.clone());

    try_join!(
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>()
        ),
        background::task(state)
    )?;
    Ok(())
}

struct AppError(pub Error);

impl From<Error> for AppError {
    fn from(err: Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            Error::FailedToConnectServer => StatusCode::UNAUTHORIZED,
            _ => StatusCode::BAD_REQUEST,
        };
        (status, Json(self.0)).into_response()
    }
}
