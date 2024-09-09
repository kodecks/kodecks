use crate::scene::GlobalState;
use bevy::{ecs::world::Command, prelude::*};
use futures::channel::mpsc::Receiver;
use kodecks::{action::Action, game::LocalGameState, player::PlayerId, profile::GameProfile};
use kodecks_server::message::{self, Input};
use std::sync::Mutex;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ServerEvent>()
            .init_resource::<Server>()
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup)
            .add_systems(Update, recv_events);
    }
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Session>();
}

#[derive(Resource)]
pub struct Server {
    server: kodecks_server::Server,
    event_recv: Receiver<kodecks_server::message::Output>,
}

impl Default for Server {
    fn default() -> Self {
        let (event_send, event_recv) = futures::channel::mpsc::channel(256);
        let event_send = Mutex::new(event_send);
        let server = kodecks_server::Server::new(move |event| {
            event_send.lock().unwrap().try_send(event).unwrap();
        });
        Self { server, event_recv }
    }
}

impl Server {
    pub fn create_session(&mut self, profile: GameProfile) {
        self.server
            .handle_input(Input::Command(message::Command::CreateSession { profile }));
    }
}

#[derive(Resource)]
struct Session {
    id: u32,
    player: PlayerId,
}

fn recv_events(
    mut commands: Commands,
    mut server: ResMut<Server>,
    mut events: EventWriter<ServerEvent>,
) {
    while let Ok(Some(event)) = server.event_recv.try_next() {
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
            if let Some(mut conn) = world.get_resource_mut::<Server>() {
                conn.server
                    .handle_input(Input::SessionCommand(message::SessionCommand {
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
