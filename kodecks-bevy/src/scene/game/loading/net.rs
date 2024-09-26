use bevy::{prelude::*, tasks::IoTaskPool};
use bevy_mod_reqwest::reqwest::Client;
use futures_util::{SinkExt, TryStreamExt};
use reqwest_websocket::{Message, RequestBuilderExt};
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

pub fn start_session(session: Res<ServerSession>) {
    let mut url = session.url.join("/ws").unwrap();
    url.query_pairs_mut().append_pair("token", &session.token);

    IoTaskPool::get()
        .spawn(async move {
            async_compat::Compat::new(async {
                let response = Client::default().get(url).upgrade().send().await.unwrap();
                let mut websocket = response.into_websocket().await.unwrap();

                websocket
                    .send(Message::Text("Hello, World".into()))
                    .await
                    .unwrap();

                while let Some(message) = websocket.try_next().await.unwrap() {
                    if let Message::Text(text) = message {
                        println!("received: {text}")
                    }
                }
            })
            .await;
        })
        .detach();
}
