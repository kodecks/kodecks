use super::{
    board, event,
    mode::{GameMode, GameModeKind},
    server::{self, ServerConnection, ServerError},
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
use bevy_mod_picking::prelude::*;
use kodecks::{
    error::Error,
    player::PlayerConfig,
    profile::{BotConfig, GameProfile},
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
            .add_systems(
                OnTransition {
                    exited: GlobalState::GameInit,
                    entered: GlobalState::MenuMain,
                },
                cleanup_loading_screen,
            )
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
                    receive_error.run_if(resource_exists_and_changed::<ServerError>),
                )
                    .run_if(in_state(GlobalState::GameInit)),
            );
    }
}

#[derive(Debug, Default, States, Clone, Eq, PartialEq, Hash)]
enum GameLoadingState {
    #[default]
    Idle,
    BotMatch,
    RandomMatch,
    Error(Error),
}

#[derive(Component)]
struct UiRoot;

#[derive(Component)]
struct UiButton;

#[derive(Component)]
struct LoadingMessage;

fn receive_error(
    mut next_loading_state: ResMut<NextState<GameLoadingState>>,
    error: Res<ServerError>,
) {
    next_loading_state.set(GameLoadingState::Error(error.0.clone()));
}

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
    asset_server: Res<AssetServer>,
) {
    next_spinner_state.set(SpinnerState::On);
    next_loading_state.set(GameLoadingState::Idle);

    let slicer = TextureSlicer {
        border: BorderRect::square(5.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    };
    let button = asset_server.load("ui/button-red.png");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 1.0).into(),
                ..default()
            },
            UiRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(75.0),
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            text: Text {
                                sections: vec![TextSection {
                                    style: TextStyle {
                                        color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                                        ..translator.style(TextPurpose::Loading)
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

            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(25.0),
                            justify_content: JustifyContent::Start,
                            align_content: AlignContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(20.),
                            ..default()
                        },
                        ..default()
                    },
                    UiButton,
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            ImageBundle {
                                style: Style {
                                    width: Val::Px(280.),
                                    height: Val::Px(50.),
                                    padding: UiRect::all(Val::Px(15.)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                image: button.clone().into(),
                                ..default()
                            },
                            ImageScaleMode::Sliced(slicer.clone()),
                            On::<Pointer<Click>>::commands_mut(move |_, commands| {
                                commands.add(move |w: &mut World| {
                                    if let Some(mut next_state) =
                                        w.get_resource_mut::<NextState<GlobalState>>()
                                    {
                                        println!("Go to main menu");
                                        next_state.set(GlobalState::MenuMain);
                                    }
                                });
                            }),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    translator.get("loading-button-cancel"),
                                    translator.style(TextPurpose::Button),
                                ),
                                Label,
                            ));
                        });
                });
        });
}

fn update_loading_message(
    mut query: Query<&mut Text, With<LoadingMessage>>,
    mut button_query: Query<&mut Visibility, With<UiButton>>,
    translator: Res<Translator>,
    loading_state: Res<State<GameLoadingState>>,
) {
    let mut text = query.single_mut();
    let message = match loading_state.get() {
        GameLoadingState::RandomMatch => translator.get("loading-message-finding-player"),
        GameLoadingState::Error(error) => translator.get(error.clone()),
        _ => "".into(),
    };
    text.sections[0].value = message.to_string();

    let visible = match loading_state.get() {
        GameLoadingState::RandomMatch | GameLoadingState::Error(_) => Visibility::Visible,
        _ => Visibility::Hidden,
    };
    button_query.iter_mut().for_each(|mut visibility| {
        *visibility = visible;
    });
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
                debug: None,
                players: vec![
                    PlayerConfig {
                        deck: mode.player_deck.clone(),
                    },
                    PlayerConfig {
                        deck: bot_deck.clone(),
                    },
                ],
                bots: vec![BotConfig { player: 1 }],
                rng_seed: Some(hasher.finish()),
            };

            let log_id = format!(
                "{}-{}",
                chrono::Local::now().format("%Y%m%d%H%M%S"),
                nanoid::nanoid!()
            );
            let mut conn = ServerConnection::new_local();
            conn.send(Input::Command(Command::CreateGame { log_id, profile }));
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
