use super::{
    board, event,
    mode::{GameMode, GameModeKind},
    server::{self, Server},
};
use crate::scene::{config::GlobalConfig, spinner::SpinnerState, GlobalState};
use bevy::{prelude::*, utils::HashMap};
use bevy_mod_reqwest::*;
use kodecks::{
    player::PlayerConfig,
    profile::{BotConfig, DebugConfig, DebugFlags, GameProfile},
};
use kodecks_engine::{
    message::{Command, Input},
    Connection,
};

mod net;

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
                    net::start_session.run_if(resource_added::<net::ServerSession>),
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
    mut conn: ResMut<Server>,
    mut next_loading_state: ResMut<NextState<GameLoadingState>>,
    mode: Res<GameMode>,
    mut client: BevyReqwest,
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
            conn.send(Input::Command(Command::CreateSession { profile }));
            next_loading_state.set(GameLoadingState::BotMatch);
        }
        GameModeKind::RandomMatch { server } => {
            let url = server.join("login").unwrap();
            let request = client
                .post(url.clone())
                .json(&HashMap::<String, u32>::new())
                .build()
                .unwrap();

            client
                .send(request)
                .on_response(
                    |trigger: Trigger<ReqwestResponseEvent>,
                     mut commands: Commands,
                     config: Res<GlobalConfig>| {
                        let response = trigger.event();
                        let data = response.deserialize_json::<net::LoginResponse>().ok();
                        let status = response.status();
                        info!("code: {status}, data: {data:?}");
                        if let Some(data) = data {
                            commands.insert_resource(net::ServerSession {
                                url: config.server.clone(),
                                token: data.token,
                            });
                        }
                    },
                )
                .on_error(|trigger: Trigger<ReqwestErrorEvent>| {
                    let e = &trigger.event().0;
                    error!("error: {e:?}");
                });
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
