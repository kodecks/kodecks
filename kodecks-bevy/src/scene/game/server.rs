use crate::scene::GlobalState;
use bevy::{ecs::world::Command, prelude::*};
use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};
use futures::channel::mpsc::{Receiver, Sender};
use kodecks::{
    action::{Action, PlayerAvailableActions},
    env::LocalEnvironment,
    game::Game,
    log::LogAction,
    player::PlayerId,
    profile::GameProfile,
};
use kodecks_bot::{Bot, DefaultBot};
use kodecks_catalog::CATALOG;
use serde_tuple::{Deserialize_tuple, Serialize_tuple};

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ServerEvent>()
            .add_systems(OnEnter(GlobalState::GameInit), init)
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup)
            .add_systems(Update, recv_events.run_if(resource_exists::<ServerChannel>))
            .add_systems(
                Update,
                run_server
                    .run_if(resource_exists::<Server>.and_then(resource_exists::<ServerChannel>)),
            );
    }
}

fn init(mut commands: Commands) {
    commands.insert_resource(ServerChannel::default());
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<ServerChannel>();
}

#[derive(Resource)]
pub struct Server {
    game: Game,
    available_actions: Option<PlayerAvailableActions>,
    user_player: PlayerId,
    player_in_action: PlayerId,
}

impl Server {
    pub fn new(profile: GameProfile) -> Self {
        let game = Game::new(profile, &CATALOG);
        let player_in_action = game.env().state.players.player_in_turn().id;
        Self {
            game,
            available_actions: None,
            user_player: PlayerId::new("player1"),
            player_in_action,
        }
    }
}

pub struct SendCommand(pub Action);

impl Command for SendCommand {
    fn apply(self, world: &mut World) {
        if let Some(mut channel) = world.get_resource_mut::<ServerChannel>() {
            let _ = channel.cmd_send.try_send(self.0);
        }
    }
}

#[derive(Event, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct ServerEvent {
    pub env: LocalEnvironment,
    pub logs: Vec<LogAction>,
    pub available_actions: Option<PlayerAvailableActions>,
}

#[derive(Resource)]
struct ServerChannel {
    cmd_send: Sender<Action>,
    cmd_recv: Receiver<Action>,
    event_send: Sender<ServerEvent>,
    event_recv: Receiver<ServerEvent>,
}

impl Default for ServerChannel {
    fn default() -> Self {
        let (cmd_send, cmd_recv) = futures::channel::mpsc::channel(256);
        let (event_send, event_recv) = futures::channel::mpsc::channel(256);
        Self {
            cmd_send,
            cmd_recv,
            event_send,
            event_recv,
        }
    }
}

fn recv_events(mut channel: ResMut<ServerChannel>, mut events: EventWriter<ServerEvent>) {
    while let Ok(Some(event)) = channel.event_recv.try_next() {
        events.send(event);
    }
}

fn run_server(
    mut server: ResMut<Server>,
    mut channel: ResMut<ServerChannel>,
    mut runner: AsyncTaskRunner<Option<Action>>,
) {
    let mut action = None;

    if server
        .available_actions
        .as_ref()
        .map_or(false, |actions| !actions.actions.is_empty())
    {
        if server.player_in_action == server.user_player {
            if let Ok(Some(player_action)) = channel.cmd_recv.try_next() {
                action = Some(player_action);
            } else {
                return;
            }
        } else {
            match runner.poll() {
                AsyncTaskStatus::Finished(bot_action) => {
                    action = bot_action;
                }
                AsyncTaskStatus::Idle => {}
                _ => {
                    return;
                }
            }
        }
    }

    let player = server.player_in_action;
    let report = server.game.tick(player, action);
    server
        .available_actions
        .clone_from(&report.available_actions);

    if let Some(available_actions) = &report.available_actions {
        server.player_in_action = available_actions.player;
    }

    if let Some(actions) = &report.available_actions {
        if actions.player != server.user_player && !actions.actions.is_empty() {
            let env = server.game.env().clone();
            let report = report.clone();
            if let Some(available_actions) = report.available_actions {
                runner.start(async move {
                    let mut bot = DefaultBot::builder().build();
                    bot.compute_best_action(env, &available_actions)
                });
            }
        }
    }

    let _ = channel.event_send.try_send(ServerEvent {
        env: server.game.env().local(server.user_player),
        logs: report.logs,
        available_actions: report
            .available_actions
            .filter(|actions| actions.player == server.user_player),
    });
}
