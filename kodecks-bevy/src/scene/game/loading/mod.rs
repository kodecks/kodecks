use super::{
    board, event,
    mode::{GameMode, GameModeKind},
    server::{self, ServerConnection},
};
use crate::scene::{spinner::SpinnerState, GlobalState};
use bevy::prelude::*;
use kodecks::{
    player::PlayerConfig,
    profile::{BotConfig, DebugConfig, DebugFlags, GameProfile},
};
use kodecks_engine::{
    message::{Command, Input},
    Connection,
};

pub struct GameLoadingPlugin;

impl Plugin for GameLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameLoadingState>()
            .add_plugins(server::ServerPlugin)
            .add_plugins(board::BoardPlugin)
            .add_plugins(event::EventPlugin)
            .add_systems(OnEnter(GlobalState::GameInit), init_loading_screen)
            .add_systems(OnExit(GlobalState::GameLoading), cleanup_loading_screen)
            .add_systems(
                Update,
                (finish_load.run_if(resource_exists::<board::Environment>),)
                    .run_if(in_state(GlobalState::GameLoading)),
            )
            .add_systems(
                Update,
                (
                    init_game_mode.run_if(in_state(GameLoadingState::Idle)),
                    wait_env.run_if(resource_exists::<board::Environment>),
                )
                    .run_if(in_state(GlobalState::GameInit)),
            );
    }
}

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
enum GameLoadingState {
    #[default]
    Idle,
    BotMatch,
    RandomMatch,
}

#[derive(Component)]
struct UiRoot;

fn finish_load(
    time: Res<Time>,
    mut query: Query<&mut BackgroundColor, With<UiRoot>>,
    mut next_state: ResMut<NextState<GlobalState>>,
) {
    let mut color = query.single_mut();
    let alpha = color.0.alpha();
    if alpha > 0.0 {
        color.0.set_alpha(alpha - time.delta_seconds());
    } else {
        next_state.set(GlobalState::GameMain);
    }
}

fn wait_env(mut next_state: ResMut<NextState<GlobalState>>) {
    next_state.set(GlobalState::GameLoading);
}

fn init_loading_screen(
    mut commands: Commands,
    mut next_spinner_state: ResMut<NextState<SpinnerState>>,
    mut next_loading_state: ResMut<NextState<GameLoadingState>>,
) {
    next_spinner_state.set(SpinnerState::On);
    next_loading_state.set(GameLoadingState::Idle);

    commands.spawn((
        NodeBundle {
            z_index: ZIndex::Global(2),
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.)),
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 1.0).into(),
            ..default()
        },
        UiRoot,
    ));
}

fn init_game_mode(
    mut commands: Commands,
    mut next_loading_state: ResMut<NextState<GameLoadingState>>,
    mode: Res<GameMode>,
) {
    match &mode.kind {
        GameModeKind::BotMatch { bot_deck } => {
            let profile = GameProfile {
                regulation: mode.regulation.clone(),
                debug: DebugConfig {
                    flags: DebugFlags::DEBUG_COMMAND,
                    ..Default::default()
                },
                players: vec![
                    PlayerConfig {
                        id: 1,
                        deck: mode.player_deck.clone(),
                    },
                    PlayerConfig {
                        id: 2,
                        deck: bot_deck.clone(),
                    },
                ],
                bots: vec![BotConfig { player: 2 }],
            };

            let mut conn = ServerConnection::new_local();
            conn.send(Input::Command(Command::CreateSession { profile }));
            commands.insert_resource(conn);

            next_loading_state.set(GameLoadingState::BotMatch);
        }
        GameModeKind::RandomMatch { server } => {
            let mut conn = ServerConnection::new_websocket(server.clone());
            conn.send(Input::Command(Command::StartRandomMatch { deck: mode.player_deck.clone() }));
            commands.insert_resource(conn);

            next_loading_state.set(GameLoadingState::RandomMatch);
        }
    }
}

fn cleanup_loading_screen(
    mut commands: Commands,
    query: Query<Entity, With<UiRoot>>,
    mut next_spinner_state: ResMut<NextState<SpinnerState>>,
) {
    next_spinner_state.set(SpinnerState::Off);

    query.iter().for_each(|entity| {
        commands.add(DespawnRecursive { entity });
    });
}
