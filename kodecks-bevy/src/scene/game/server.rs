use crate::scene::GlobalState;
use bevy::{ecs::world::Command, prelude::*};
use kodecks::{action::Action, game::LocalGameState, player::PlayerId};
use kodecks_server::{
    local::LocalServer,
    message::{self, Input},
    Connection,
};

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

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Server(LocalServer);

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
