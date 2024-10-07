use super::card::{Card, CardSize, CardState, CARD_WIDTH};
use super::event::PlayerEventFinished;
use crate::scene::game::board::{self, Board};
use crate::scene::game::event::{LogEvent, LogEventQueue};
use crate::scene::GlobalState;
use bevy::animation::AnimationTargetId;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::Command;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashSet;
use kodecks::player::PlayerZone;
use kodecks::target::Target;
use kodecks::zone::Zone;
use std::f32::consts::PI;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AnimationState>()
            .add_systems(OnEnter(GlobalState::GameLoading), init)
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup)
            .add_systems(
                Update,
                (
                    update_card_state.run_if(in_state(CardState::Animating)),
                    update_animation_list.run_if(in_state(AnimationState::Sync)),
                    (start_attack_animation.pipe(start_move_animation))
                        .run_if(on_event::<PlayerEventFinished>()),
                )
                    .run_if(
                        in_state(GlobalState::GameMain)
                            .or_else(in_state(GlobalState::GameResult))
                            .or_else(in_state(GlobalState::GameLoading)),
                    ),
            )
            .add_systems(
                OnEnter(CardState::Loaded),
                (start_attack_animation.pipe(start_move_animation)).run_if(
                    in_state(GlobalState::GameMain).or_else(in_state(GlobalState::GameLoading)),
                ),
            );
    }
}

fn init(mut commands: Commands, mut next_state: ResMut<NextState<AnimationState>>) {
    commands.insert_resource(AnimationList::default());
    next_state.set(AnimationState::default());
}

fn cleanup(mut commands: Commands, mut next_state: ResMut<NextState<AnimationState>>) {
    commands.remove_resource::<AnimationList>();
    next_state.set(AnimationState::default());
}

#[derive(Resource, Default)]
struct AnimationList {
    pub sync_targets: Vec<Entity>,
}

pub enum RegisterAnimation {
    Sync(Entity),
    Async,
}

impl Command for RegisterAnimation {
    fn apply(self, world: &mut World) {
        let mut list = world.get_resource_or_insert_with(AnimationList::default);

        if let RegisterAnimation::Sync(entity) = self {
            list.sync_targets.push(entity);
        }

        world
            .get_resource_mut::<NextState<AnimationState>>()
            .unwrap()
            .set(AnimationState::Sync);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AnimationState {
    #[default]
    Idle,
    Sync,
}

fn update_animation_list(
    mut list: ResMut<AnimationList>,
    query: Query<&AnimationPlayer>,
    mut next_state: ResMut<NextState<AnimationState>>,
) {
    list.sync_targets.retain(|target| {
        query
            .get(*target)
            .ok()
            .map_or(false, |player| !player.all_finished())
    });

    next_state.set(if list.sync_targets.is_empty() {
        AnimationState::Idle
    } else {
        AnimationState::Sync
    });
}

fn update_card_state(
    anim_state: Res<State<AnimationState>>,
    mut next_state: ResMut<NextState<CardState>>,
) {
    if *anim_state == AnimationState::Idle {
        next_state.set(CardState::Ready);
    }
}

type CardAnimationQuery<'world, 'state, 'a> = Query<
    'world,
    'state,
    (
        &'a mut Card,
        Entity,
        &'a Name,
        &'a mut AnimationPlayer,
        &'a mut Handle<AnimationGraph>,
        &'a Transform,
    ),
>;

#[derive(SystemParam)]
pub struct AnimationResources<'w> {
    env: Res<'w, board::Environment>,
    board: Res<'w, Board>,
    animations: ResMut<'w, Assets<AnimationClip>>,
    graphs: ResMut<'w, Assets<AnimationGraph>>,
    log_events: ResMut<'w, LogEventQueue>,
    next_state: ResMut<'w, NextState<CardState>>,
}

fn start_attack_animation(
    mut res: AnimationResources,
    mut commands: Commands,
    mut query: CardAnimationQuery,
    camera: Query<&Transform, With<Camera>>,
) -> bool {
    if matches!(res.log_events.front(), Some(LogEvent::Attack { .. })) {
        let camera_pos = camera.single().translation;
        if let Some(LogEvent::Attack { attacker, target }) = res.log_events.pop_front() {
            let mut blocker_translation = Vec3::INFINITY;

            query
                .iter_mut()
                .filter(|(card, _, _, _, _, _)| card.id == attacker)
                .for_each(
                    |(card, entity, root_name, mut anim, mut graph, transform)| {
                        let mut animation = AnimationClip::default();

                        let mut target_transform = if let Target::Card(target) = target {
                            let zone = res.env.find_zone(target).unwrap();
                            Transform::from_translation(
                                res.board
                                    .get_zone_transform(target, zone, res.env.player, camera_pos)
                                    .map(|t| t.translation)
                                    .unwrap(),
                            )
                        } else if target == Target::Player(res.env.player) {
                            Transform::from_xyz(0.0, 2.0, 2.4)
                        } else {
                            Transform::from_xyz(0.0, 2.0, -2.2)
                        };

                        blocker_translation =
                            (target_transform.translation - transform.translation) / 4.0;
                        target_transform.translation.y = 0.3;

                        let mut target =
                            transform.looking_at(target_transform.translation, Vec3::Y);
                        let zone = res.env.find_zone(card.id).unwrap();
                        if zone.player != res.env.player {
                            target.rotate_y(PI);
                        }
                        let target_rotation = target.rotation;

                        animation.add_curve_to_target(
                            AnimationTargetId::from_names([root_name.clone()].iter()),
                            VariableCurve {
                                keyframe_timestamps: vec![0.0, 0.1, 0.2],
                                keyframes: Keyframes::Translation(vec![
                                    transform.translation,
                                    target_transform.translation,
                                    transform.translation,
                                ]),
                                interpolation: Interpolation::Linear,
                            },
                        );

                        animation.add_curve_to_target(
                            AnimationTargetId::from_names([root_name.clone()].iter()),
                            VariableCurve {
                                keyframe_timestamps: vec![0.0, 0.01, 0.19, 0.2],
                                keyframes: Keyframes::Rotation(vec![
                                    transform.rotation,
                                    target_rotation,
                                    target_rotation,
                                    transform.rotation,
                                ]),
                                interpolation: Interpolation::Linear,
                            },
                        );

                        let (anim_graph, animation_index) =
                            AnimationGraph::from_clip(res.animations.add(animation));
                        *graph = res.graphs.add(anim_graph);
                        *anim = AnimationPlayer::default();
                        anim.play(animation_index);
                        commands.add(RegisterAnimation::Sync(entity));
                    },
                );

            query
                .iter_mut()
                .filter(|(card, _, _, _, _, _)| Target::Card(card.id) == target)
                .for_each(|(_, entity, root_name, mut anim, mut graph, transform)| {
                    let mut animation = AnimationClip::default();
                    let target_transform =
                        transform.with_translation(transform.translation + blocker_translation);

                    animation.add_curve_to_target(
                        AnimationTargetId::from_names([root_name.clone()].iter()),
                        VariableCurve {
                            keyframe_timestamps: vec![0.0, 0.07, 0.1, 0.2, 0.25],
                            keyframes: Keyframes::Translation(vec![
                                transform.translation,
                                transform.translation,
                                target_transform.translation,
                                transform.translation,
                                transform.translation,
                            ]),
                            interpolation: Interpolation::Linear,
                        },
                    );

                    let (anim_graph, animation_index) =
                        AnimationGraph::from_clip(res.animations.add(animation));
                    *graph = res.graphs.add(anim_graph);
                    *anim = AnimationPlayer::default();
                    anim.play(animation_index);
                    commands.add(RegisterAnimation::Sync(entity));
                });
        }
        return true;
    }
    false
}

fn start_move_animation(
    In(result): In<bool>,
    mut commands: Commands,
    mut res: AnimationResources,
    mut query: CardAnimationQuery,
    size_query: Query<(&Name, &Transform, &CardSize)>,
    camera: Query<&Transform, With<Camera>>,
    children: Query<&Children>,
) {
    if !result {
        let camera_pos = camera.single().translation;

        let mut moved = HashSet::new();
        while matches!(res.log_events.front(), Some(LogEvent::Moved { .. })) {
            if let Some(LogEvent::Moved { card }) = res.log_events.pop_front() {
                moved.insert(card);
            }
        }

        query.iter_mut().for_each(
            |(card, entity, root_name, mut anim, mut graph, transform)| {
                let (zone, log) = if let Ok(zone) = res.env.find_zone(card.id) {
                    (zone, moved.contains(&card.id))
                } else {
                    (
                        PlayerZone {
                            player: card.owner,
                            zone: Zone::Deck,
                        },
                        false,
                    )
                };

                let mut animation = AnimationClip::default();

                let keyframe_timestamps = vec![0.0, 0.1, 0.2];

                if let Some(target_transform) =
                    res.board
                        .get_zone_transform(card.id, zone, res.env.player, camera_pos)
                {
                    animation.add_curve_to_target(
                        AnimationTargetId::from_names([root_name.clone()].iter()),
                        VariableCurve {
                            keyframe_timestamps: keyframe_timestamps.clone(),
                            keyframes: Keyframes::Translation(vec![
                                transform.translation,
                                target_transform.translation,
                                target_transform.translation,
                            ]),
                            interpolation: Interpolation::Linear,
                        },
                    );
                    animation.add_curve_to_target(
                        AnimationTargetId::from_names([root_name.clone()].iter()),
                        VariableCurve {
                            keyframe_timestamps: keyframe_timestamps.clone(),
                            keyframes: Keyframes::Rotation(vec![
                                transform.rotation,
                                target_transform.rotation,
                                target_transform.rotation,
                            ]),
                            interpolation: Interpolation::Linear,
                        },
                    );

                    animation.add_curve_to_target(
                        AnimationTargetId::from_names([root_name.clone()].iter()),
                        VariableCurve {
                            keyframe_timestamps: keyframe_timestamps.clone(),
                            keyframes: Keyframes::Scale(vec![
                                transform.scale,
                                target_transform.scale,
                                target_transform.scale,
                            ]),
                            interpolation: Interpolation::Linear,
                        },
                    );
                }

                for child in children.iter_descendants(entity) {
                    if let Ok((name, transform, size)) = size_query.get(child) {
                        let frame_scale = if zone.zone == Zone::Field {
                            if *size == CardSize::Large {
                                0.0
                            } else {
                                1.0
                            }
                        } else if *size == CardSize::Large {
                            1.0
                        } else {
                            0.0
                        };
                        animation.add_curve_to_target(
                            AnimationTargetId::from_names([root_name.clone(), name.clone()].iter()),
                            VariableCurve {
                                keyframe_timestamps: vec![0.0, 0.1, 0.2],
                                keyframes: Keyframes::Scale(vec![
                                    transform.scale,
                                    Vec3::splat(1.0 / CARD_WIDTH * frame_scale),
                                    Vec3::splat(1.0 / CARD_WIDTH * frame_scale),
                                ]),
                                interpolation: Interpolation::Linear,
                            },
                        );
                    }
                }

                let (anim_graph, animation_index) =
                    AnimationGraph::from_clip(res.animations.add(animation));
                *graph = res.graphs.add(anim_graph);
                *anim = AnimationPlayer::default();
                anim.play(animation_index);

                commands.add(if log {
                    RegisterAnimation::Sync(entity)
                } else {
                    RegisterAnimation::Async
                });
            },
        );
    }
    res.next_state.set(CardState::Animating);
}
