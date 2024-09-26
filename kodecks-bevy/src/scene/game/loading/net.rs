use bevy::prelude::*;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Resource)]
pub struct ServerSession {
    pub url: Url,
    pub token: String,
}
