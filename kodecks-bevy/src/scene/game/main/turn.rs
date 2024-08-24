use crate::scene::{
    game::{board::Environment, event::TurnChanged},
    translator::{TextPurpose, Translator},
    GlobalState,
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<FadeState>()
            .add_systems(
                Update,
                (
                    trigger.run_if(on_event::<TurnChanged>()),
                    fade.run_if(in_state(FadeState::FadeIn).or_else(in_state(FadeState::FadeOut))),
                ),
            )
            .add_systems(OnEnter(GlobalState::GameLoading), init)
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup);
    }
}

#[derive(Component)]
struct UiRoot;

#[derive(Component)]
struct TurnOverlay;

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
enum FadeState {
    #[default]
    Idle,
    FadeIn,
    FadeOut,
}

fn trigger(
    env: Res<Environment>,
    translator: Res<Translator>,
    mut event: EventReader<TurnChanged>,
    mut query: Query<&mut Text, With<TurnOverlay>>,
    mut next_state: ResMut<NextState<FadeState>>,
) {
    if let Some(TurnChanged(player)) = event.read().next() {
        query.single_mut().sections[0].value = if env.player == *player {
            translator.get("your-turn")
        } else {
            translator.get("opponents-turn")
        }
        .to_string();
        next_state.set(FadeState::FadeIn);
    }
}

fn fade(
    mut query: Query<&mut Text, With<TurnOverlay>>,
    state: Res<State<FadeState>>,
    mut next_state: ResMut<NextState<FadeState>>,
    time: Res<Time>,
) {
    let color = &mut query.single_mut().sections[0].style.color;
    if *state == FadeState::FadeIn {
        color.set_alpha(color.alpha() + time.delta_seconds() * 2.0);
        if color.alpha() >= 1.0 {
            next_state.set(FadeState::FadeOut);
        }
    } else if *state == FadeState::FadeOut {
        color.set_alpha(color.alpha() - time.delta_seconds() * 2.0);
        if color.alpha() <= 0.0 {
            next_state.set(FadeState::Idle);
        }
    }
}

fn init(mut commands: Commands, translator: Res<Translator>) {
    commands
        .spawn((
            NodeBundle {
                z_index: ZIndex::Global(2),
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.)),
                    ..default()
                },
                ..default()
            },
            Pickable::IGNORE,
            UiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            style: TextStyle {
                                color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                                ..translator.style(TextPurpose::Title)
                            },
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    z_index: ZIndex::Global(1),
                    ..default()
                },
                Pickable::IGNORE,
                Label,
                TurnOverlay,
            ));
        });
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<UiRoot>>) {
    query.iter().for_each(|entity| {
        commands.add(DespawnRecursive { entity });
    });
}
