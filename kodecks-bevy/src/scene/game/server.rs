use crate::scene::{spinner::SpinnerState, GlobalState};
use bevy::{ecs::world::Command, prelude::*};
use futures::{
    channel::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
    select, FutureExt, StreamExt,
};
use futures_util::SinkExt;
use k256::{ecdsa::signature::SignerMut, schnorr::SigningKey};
use kodecks::{action::Action, env::LocalGameState, error::Error};
use kodecks_engine::{
    login::{LoginRequest, LoginResponse, LoginType},
    message::{self, GameEventKind, Input, Output},
    Connection,
};
use reqwest_websocket::{CloseCode, RequestBuilderExt, WebSocket};
use semver::Version;
use std::pin::pin;
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
    #[cfg(not(target_family = "wasm"))]
    Local(kodecks_engine::local::LocalEngine),
    #[cfg(target_family = "wasm")]
    Local(kodecks_engine::worker::WebWorkerEngine),
    WebSocket(WebSocketEngine),
}

impl ServerConnection {
    pub fn new_local() -> Self {
        Self::Local(Default::default())
    }

    pub fn new_websocket(server: Url, key: SigningKey) -> Self {
        Self::WebSocket(WebSocketEngine::new(server, key))
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

#[derive(Resource, Clone, Deref)]
pub struct ServerError(pub Error);

pub struct WebSocketEngine {
    command_send: Sender<Input>,
    event_recv: Receiver<Output>,
    close_send: Option<oneshot::Sender<()>>,
    #[cfg(not(target_family = "wasm"))]
    task: bevy::tasks::Task<()>,
}

impl WebSocketEngine {
    pub fn new(server: Url, key: SigningKey) -> Self {
        let (mut event_send, event_recv) = mpsc::channel(256);
        let (command_send, command_recv) = mpsc::channel(256);
        let (close_send, close_recv) = oneshot::channel();

        let task = bevy::tasks::IoTaskPool::get().spawn(async move {
            #[cfg(target_family = "wasm")]
            if let Err(err) =
                connect(server, command_recv, event_send.clone(), close_recv, key).await
            {
                error!("Websocket error: {}", err);
                let err: Error = if let Some(err) = err.downcast_ref::<Error>() {
                    err.clone()
                } else {
                    Error::FailedToConnectServer
                };
                let _ = event_send.try_send(Output::Error(err));
            }

            #[cfg(not(target_family = "wasm"))]
            async_compat::Compat::new(async {
                if let Err(err) =
                    connect(server, command_recv, event_send.clone(), close_recv, key).await
                {
                    error!("Websocket error: {}", err);
                    let err: Error = if let Some(err) = err.downcast_ref::<Error>() {
                        err.clone()
                    } else {
                        Error::FailedToConnectServer
                    };
                    let _ = event_send.try_send(Output::Error(err));
                }
            })
            .await;
        });

        #[cfg(target_family = "wasm")]
        task.detach();

        Self {
            command_send,
            event_recv,
            close_send: Some(close_send),
            #[cfg(not(target_family = "wasm"))]
            task,
        }
    }
}

impl Drop for WebSocketEngine {
    fn drop(&mut self) {
        self.command_send.close_channel();
        self.event_recv.close();
        let _ = self.close_send.take().unwrap().send(());
        #[cfg(not(target_family = "wasm"))]
        bevy::tasks::block_on(&mut self.task);
    }
}

async fn connect(
    server: Url,
    mut command_recv: Receiver<Input>,
    mut event_send: Sender<Output>,
    mut close_recv: oneshot::Receiver<()>,
    key: SigningKey,
) -> anyhow::Result<()> {
    let socket = connect_websocket(server, key).fuse();
    let mut socket = pin!(socket);
    let websocket = select! {
        socket = socket => {
            socket?
        },
        _ = close_recv => {
            return Ok(());
        }
    };

    let mut websocket = websocket.fuse();
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
            _ = close_recv => {
                break;
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

async fn connect_websocket(server: Url, key: SigningKey) -> anyhow::Result<WebSocket> {
    let url = server.join("login")?;

    let client_version: Version = env!("CARGO_PKG_VERSION").parse().unwrap();
    let pubkey = key.verifying_key();
    let client = reqwest::Client::new();

    let res = client
        .post(url.clone())
        .json(&LoginRequest {
            client_version: client_version.clone(),
            ty: LoginType::PubkeyChallenge { pubkey: *pubkey },
        })
        .send()
        .await?;
    let res: LoginResponse = if res.status().is_success() {
        res.json().await?
    } else {
        let err: Error = res.json().await?;
        return Err(err.into());
    };

    let challenge = if let LoginResponse::Challenge { challenge } = res {
        challenge
    } else {
        return Err(anyhow::anyhow!("Unexpected response"));
    };

    let mut key: SigningKey = key.clone();
    let signature = key.sign(challenge.as_bytes());
    let res = client
        .post(url.clone())
        .json(&LoginRequest {
            client_version,
            ty: LoginType::PubkeyResponse {
                pubkey: *pubkey,
                signature,
            },
        })
        .send()
        .await?;

    let res: LoginResponse = if res.status().is_success() {
        res.json().await?
    } else {
        let err: Error = res.json().await?;
        return Err(err.into());
    };

    let token = if let LoginResponse::Session { token } = res {
        token
    } else {
        return Err(anyhow::anyhow!("Unexpected response"));
    };

    let mut url = url.join("/ws")?;
    url.query_pairs_mut().append_pair("token", &token);

    let response = client.get(url).upgrade().send().await?;
    Ok(response.into_websocket().await?)
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
    mut next_spinner_state: ResMut<NextState<SpinnerState>>,
) {
    while let Some(event) = server.recv() {
        commands.remove_resource::<ServerError>();

        match event {
            Output::GameEvent(event) => match event.event {
                message::GameEventKind::Created { .. } => {
                    commands.insert_resource(Session {
                        id: event.game_id,
                        player: event.player,
                    });
                }
                GameEventKind::StateUpdated { state } => {
                    events.send(ServerEvent(state));
                    next_spinner_state.set(SpinnerState::Off);
                }
                GameEventKind::PlayerThinking { thinking, timeout } => {
                    info!("Player {} is thinking: {:?}", thinking, timeout);
                    if thinking != event.player {
                        next_spinner_state.set(SpinnerState::On);
                    }
                }
            },
            Output::RoomEvent(event) => match event.event {
                message::RoomEventKind::Created => {
                    info!("Room created: {}", event.room_id);
                }
                message::RoomEventKind::GameRequested { guest } => {
                    info!("Game requested by {} for {}", guest, event.room_id);
                    server.send(Input::RoomCommand(message::RoomCommand {
                        room_id: event.room_id,
                        kind: message::RoomCommandKind::Approve { guest },
                    }));
                }
            },
            Output::Error(err) => {
                error!("Error: {}", err);
                commands.insert_resource(ServerError(err));
            }
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
                conn.send(Input::GameCommand(message::GameCommand {
                    game_id: id,
                    player,
                    kind: message::GameCommandKind::NextAction { action: self.0 },
                }));
            }
        }
    }
}

#[derive(Event, Clone, Deref)]
pub struct ServerEvent(LocalGameState);
