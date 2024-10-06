use super::{
    board, event,
    mode::{GameMode, GameModeKind},
    server::{self, ServerConnection},
};
use crate::{
    save_data,
    scene::{
        spinner::SpinnerState,
        translator::{TextPurpose, Translator},
        GlobalState,
    },
};
use bevy::prelude::*;
use kodecks::{
    player::PlayerConfig,
    profile::{BotConfig, DebugConfig, DebugFlags, GameProfile},
};
use kodecks_engine::{
    message::{Command, Input},
    room::{RoomConfig, RoomType},
    Connection,
};
use std::hash::{Hash, Hasher};

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
                (
                    finish_load.run_if(resource_exists::<board::Environment>),
                    update_loading_message.run_if(state_changed::<GameLoadingState>),
                )
                    .run_if(in_state(GlobalState::GameLoading)),
            )
            .add_systems(
                Update,
                (
                    init_game_mode.run_if(in_state(GameLoadingState::Idle)),
                    wait_env.run_if(resource_exists::<board::Environment>),
                    update_loading_message.run_if(state_changed::<GameLoadingState>),
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

#[derive(Component)]
struct LoadingMessage;

fn finish_load(
    time: Res<Time>,
    mut query: Query<&mut BackgroundColor, With<UiRoot>>,
    mut next_state: ResMut<NextState<GlobalState>>,
    mut next_loading_state: ResMut<NextState<GameLoadingState>>,
) {
    let mut color = query.single_mut();
    let alpha = color.0.alpha();
    if alpha > 0.0 {
        color.0.set_alpha(alpha - time.delta_seconds());
    } else {
        next_state.set(GlobalState::GameMain);
    }
    next_loading_state.set(GameLoadingState::Idle);
}

fn wait_env(mut next_state: ResMut<NextState<GlobalState>>) {
    next_state.set(GlobalState::GameLoading);
}

fn init_loading_screen(
    mut commands: Commands,
    mut next_spinner_state: ResMut<NextState<SpinnerState>>,
    mut next_loading_state: ResMut<NextState<GameLoadingState>>,
    translator: Res<Translator>,
) {
    next_spinner_state.set(SpinnerState::On);
    next_loading_state.set(GameLoadingState::Idle);

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(20.)),
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 1.0).into(),
                ..default()
            },
            UiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            style: TextStyle {
                                color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                                ..translator.style(TextPurpose::Title)
                            },
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    ..default()
                },
                Label,
                LoadingMessage,
            ));
        });
}

fn update_loading_message(
    mut query: Query<&mut Text, With<LoadingMessage>>,
    loading_state: Res<State<GameLoadingState>>,
) {
    let mut text = query.single_mut();
    let message = match loading_state.get() {
        GameLoadingState::RandomMatch => "Finding a player...",
        _ => "",
    };
    text.sections[0].value = message.to_string();
}

fn init_game_mode(
    mut commands: Commands,
    mut next_loading_state: ResMut<NextState<GameLoadingState>>,
    mode: Res<GameMode>,
    save_data: Res<save_data::SaveData>,
) {
    match &mode.kind {
        GameModeKind::BotMatch { bot_deck } => {
            let mut hasher = fnv::FnvHasher::default();
            save_data.hash(&mut hasher);
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
                rng_seed: Some(hasher.finish()),
            };

            let log_id = format!(
                "{}-{}",
                chrono::Local::now().format("%Y%m%d%H%M%S"),
                nanoid::nanoid!()
            );
            let mut conn = ServerConnection::new_local();
            conn.send(Input::Command(Command::CreateSession { log_id, profile }));
            commands.insert_resource(conn);

            next_loading_state.set(GameLoadingState::BotMatch);
        }
        GameModeKind::RandomMatch { server } => {
            let key = save_data.auth.private_key.clone();
            let mut conn = ServerConnection::new_websocket(server.clone(), key);
            conn.send(Input::Command(Command::CreateRoom {
                config: RoomConfig {
                    regulation: mode.regulation.clone(),
                    room_type: RoomType::RandomMatch,
                },
                host_player: PlayerConfig {
                    id: 0,
                    deck: mode.player_deck.clone(),
                },
            }));
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
