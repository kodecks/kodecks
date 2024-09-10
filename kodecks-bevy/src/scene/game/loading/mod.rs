use super::{
    board, event,
    server::{self, Server},
};
use crate::scene::GlobalState;
use bevy::prelude::*;
use kodecks_catalog::profile;
use kodecks_server::{
    message::{Command, Input},
    Connection,
};

pub struct GameLoadingPlugin;

impl Plugin for GameLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(server::ServerPlugin)
            .add_plugins(board::BoardPlugin)
            .add_plugins(event::EventPlugin)
            .add_systems(OnEnter(GlobalState::GameInit), init_loading_screen)
            .add_systems(OnExit(GlobalState::GameLoading), cleanup_loading_screen)
            .add_systems(
                Update,
                (finish_load.run_if(resource_exists::<board::Environment>))
                    .run_if(in_state(GlobalState::GameLoading)),
            )
            .add_systems(
                Update,
                (wait_env.run_if(resource_exists::<board::Environment>))
                    .run_if(in_state(GlobalState::GameInit)),
            );
    }
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

fn init_loading_screen(mut commands: Commands, mut conn: ResMut<Server>) {
    conn.send(Input::Command(Command::CreateSession {
        profile: profile::default_profile(),
    }));

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

fn cleanup_loading_screen(mut commands: Commands, query: Query<Entity, With<UiRoot>>) {
    query.iter().for_each(|entity| {
        commands.add(DespawnRecursive { entity });
    });
}
