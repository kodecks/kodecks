use crate::scene::GlobalState;
use bevy::{ecs::world::Command, prelude::*, utils::HashMap};
use futures::{
    channel::mpsc::{self, Receiver, Sender},
    select, StreamExt,
};
use futures_util::SinkExt;
use kodecks::{action::Action, env::LocalGameState};
use kodecks_engine::{
    message::{self, Input, Output},
    Connection,
};
use reqwest_websocket::{CloseCode, RequestBuilderExt};
use serde::Deserialize;
use url::Url;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ServerEvent>()
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup)
            .add_systems(
                Update,
                recv_events.run_if(resource_exists::<ServerConnection>),
            );
    }
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Session>();
    commands.remove_resource::<ServerConnection>();
}

#[derive(Resource)]
pub enum ServerConnection {
    #[cfg(not(target_arch = "wasm32"))]
    Local(kodecks_engine::local::LocalEngine),
    #[cfg(target_arch = "wasm32")]
    Local(kodecks_engine::worker::WebWorkerEngine),
    WebSocket(WebSocketEngine),
}

impl ServerConnection {
    pub fn new_local() -> Self {
        Self::Local(Default::default())
    }

    pub fn new_websocket(server: Url) -> Self {
        Self::WebSocket(WebSocketEngine::new(server))
    }
}

impl Connection for ServerConnection {
    fn send(&mut self, message: Input) {
        match self {
            Self::Local(conn) => conn.send(message),
            Self::WebSocket(conn) => conn.command_send.try_send(message).unwrap(),
        }
    }

    fn recv(&mut self) -> Option<Output> {
        match self {
            Self::Local(conn) => conn.recv(),
            Self::WebSocket(conn) => conn.event_recv.try_next().ok().flatten(),
        }
    }
}

pub struct WebSocketEngine {
    command_send: Sender<Input>,
    event_recv: Receiver<Output>,
    #[cfg(not(target_arch = "wasm32"))]
    task: bevy::tasks::Task<()>,
}

impl WebSocketEngine {
    pub fn new(server: Url) -> Self {
        let (event_send, event_recv) = mpsc::channel(256);
        let (command_send, command_recv) = mpsc::channel(256);

        let task = bevy::tasks::IoTaskPool::get().spawn(async move {
            #[cfg(target_arch = "wasm32")]
            if let Err(err) = connect(server, command_recv, event_send).await {
                error!("Websocket error: {}", err);
            }

            #[cfg(not(target_arch = "wasm32"))]
            async_compat::Compat::new(async {
                if let Err(err) = connect(server, command_recv, event_send).await {
                    error!("Websocket error: {}", err);
                }
            })
            .await;
        });

        #[cfg(target_arch = "wasm32")]
        task.detach();

        Self {
            command_send,
            event_recv,
            #[cfg(not(target_arch = "wasm32"))]
            task,
        }
    }
}

impl Drop for WebSocketEngine {
    fn drop(&mut self) {
        self.command_send.close_channel();
        self.event_recv.close();
        #[cfg(not(target_arch = "wasm32"))]
        bevy::tasks::block_on(&mut self.task);
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

async fn connect(
    server: Url,
    mut command_recv: Receiver<Input>,
    mut event_send: Sender<Output>,
) -> anyhow::Result<()> {
    let url = server.join("login")?;

    let client = reqwest::Client::new();
    let res: LoginResponse = client
        .post(url.clone())
        .json(&HashMap::<String, u32>::new())
        .send()
        .await?
        .json()
        .await?;

    let mut url = url.join("/ws")?;
    url.query_pairs_mut().append_pair("token", &res.token);

    let response = client.get(url).upgrade().send().await?;
    let mut websocket = response.into_websocket().await?.fuse();
    let config = bincode::config::standard();

    loop {
        select! {
            command = command_recv.next() => {
                if let Some(command) = command {
                    websocket
                        .send(reqwest_websocket::Message::Binary(bincode::encode_to_vec(&command, config)?))
                        .await?;
                } else {
                    break;
                }
            }
            message = websocket.next() => {
                if let Some(Ok(reqwest_websocket::Message::Binary(data))) = message {
                    if let Ok((event, _)) = bincode::decode_from_slice(&data, config) {
                        event_send.send(event).await?;
                    }
                } else {
                    break;
                }
            }
        }
    }

    websocket
        .into_inner()
        .close(CloseCode::Away, Some("Window closed"))
        .await?;
    info!("Connection closed");
    Ok(())
}

#[derive(Resource)]
struct Session {
    id: u32,
    player: u8,
}

fn recv_events(
    mut commands: Commands,
    mut server: ResMut<ServerConnection>,
    mut events: EventWriter<ServerEvent>,
) {
    while let Some(event) = server.recv() {
        match event {
            message::Output::SessionEvent(event) => match event.event {
                message::SessionEventKind::Created => {
                    commands.insert_resource(Session {
                        id: event.session,
                        player: event.player,
                    });
                }
                message::SessionEventKind::GameUpdated { state } => {
                    events.send(ServerEvent(state));
                }
                message::SessionEventKind::PlayerThinking { thinking } => {
                    info!("Player {} is thinking", thinking);
                }
            },
        }
    }
}

pub struct SendCommand(pub Action);

impl Command for SendCommand {
    fn apply(self, world: &mut World) {
        if let Some(session) = world.get_resource::<Session>() {
            let id = session.id;
            let player = session.player;
            if let Some(mut conn) = world.get_resource_mut::<ServerConnection>() {
                conn.send(Input::SessionCommand(message::SessionCommand {
                    session: id,
                    player,
                    kind: message::SessionCommandKind::NextAction { action: self.0 },
                }));
            }
        }
    }
}

#[derive(Event, Clone, Deref)]
pub struct ServerEvent(LocalGameState);
