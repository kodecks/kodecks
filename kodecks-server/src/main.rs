use axum::{
    middleware,
    response::Redirect,
    routing::{get, post},
    Router,
};
use http::Method;
use std::sync::Arc;
use tokio::try_join;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
};
use tracing::info;

mod app;
mod auth;
mod background;
mod login;
mod session;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(3000);
    let addr = ("0.0.0.0", port);
    info!("Listening on {}:{}", addr.0, addr.1);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let state = Arc::new(app::AppState::new());

    let authorized = Router::new()
        .route("/logout", get(login::logout))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::auth));

    let app = Router::new()
        .route(
            "/",
            get(|| async { Redirect::permanent("https://github.com/kodecks/kodecks") }),
        )
        .route("/login", post(login::login))
        .merge(authorized)
        .layer(cors)
        .layer(CompressionLayer::new())
        .with_state(state.clone());

    try_join!(axum::serve(listener, app), background::task(state))?;
    Ok(())
}
