use super::{super::GlobalState, board::Environment};
use crate::scene::translator::{TextPurpose, Translator};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use fluent_content::Request;
use kodecks::env::{EndgameReason, EndgameState};

pub struct GameResultPlugin;

impl Plugin for GameResultPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<State>()
            .add_systems(OnExit(GlobalState::GameCleanup), cleanup_loading_screen)
            .add_systems(OnEnter(GlobalState::GameResult), init)
            .add_systems(
                Update,
                (finish_result.run_if(in_state(State::Exit)))
                    .run_if(in_state(GlobalState::GameResult)),
            );
    }
}

#[derive(Component)]
struct UiRoot;

#[derive(Debug, Copy, Clone, States, Default, Eq, PartialEq, Hash)]
enum State {
    #[default]
    Await,
    Exit,
}

fn init(
    mut commands: Commands,
    env: Res<Environment>,
    translator: Res<Translator>,
    mut next_state: ResMut<NextState<State>>,
) {
    next_state.set(State::Await);

    let (message, reason) = match env.endgame {
        EndgameState::Finished { winner, reason } => {
            let attr = match reason {
                EndgameReason::Concede => "reason-concede",
                EndgameReason::DeckOut => "reason-deck-out",
                EndgameReason::LifeZero => "reason-life-zero",
                EndgameReason::SimultaneousEnd => "reason-simultaneous-end",
            };
            let request = if let Some(winner) = winner {
                if env.player == winner {
                    Request::new("result-victory")
                } else {
                    Request::new("result-defeat")
                }
            } else {
                Request::new("result-draw")
            };
            (translator.get(request), translator.get(request.attr(attr)))
        }
        _ => ("".into(), "".into()),
    };

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
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
                ..default()
            },
            Pickable {
                should_block_lower: false,
                is_hoverable: true,
            },
            On::<Pointer<Click>>::commands_mut(move |_, commands| {
                commands.add(move |w: &mut World| {
                    let mut next_state = w.resource_mut::<NextState<State>>();
                    next_state.set(State::Exit);
                });
            }),
            UiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: message.to_string(),
                            style: translator.style(TextPurpose::Result),
                        }],
                        ..Default::default()
                    },
                    z_index: ZIndex::Global(1),
                    ..default()
                },
                Label,
            ));
            parent.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: reason.to_string(),
                            style: translator.style(TextPurpose::CardName),
                        }],
                        ..Default::default()
                    },
                    z_index: ZIndex::Global(1),
                    ..default()
                },
                Label,
            ));
        });
}

fn finish_result(
    time: Res<Time>,
    mut query: Query<&mut BackgroundColor, With<UiRoot>>,
    mut next_state: ResMut<NextState<GlobalState>>,
) {
    let mut color = query.single_mut();
    let alpha = color.0.alpha();
    if alpha < 1.0 {
        color.0.set_alpha(alpha + time.delta_seconds());
    } else {
        next_state.set(GlobalState::GameCleanup);
    }
}

fn cleanup_loading_screen(mut commands: Commands, query: Query<Entity, With<UiRoot>>) {
    query.iter().for_each(|entity| {
        commands.add(DespawnRecursive { entity });
    });
}
