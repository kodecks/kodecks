use super::animation::RegisterAnimation;
use crate::{
    assets::AssetServerExt,
    scene::{
        card::Catalog,
        game::board::{Board, Environment},
        GlobalState,
    },
};
use bevy::{
    animation::{AnimationTarget, AnimationTargetId},
    prelude::*,
};

pub struct StackPlugin;

impl Plugin for StackPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<StackState>()
            .add_systems(OnEnter(GlobalState::GameLoading), init)
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup)
            .add_systems(
                Update,
                (
                    spawn_effect_stack.run_if(resource_exists_and_changed::<Environment>),
                    (update_effect_stack, animate_effect_stack)
                        .chain()
                        .run_if(in_state(StackState::Active)),
                )
                    .run_if(in_state(GlobalState::GameMain)),
            );
    }
}

fn init(mut next_state: ResMut<NextState<StackState>>) {
    next_state.set(StackState::Empty);
}

fn cleanup(
    mut commands: Commands,
    mut next_state: ResMut<NextState<StackState>>,
    query: Query<Entity, With<EffectStackItem>>,
) {
    next_state.set(StackState::Empty);
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum StackState {
    #[default]
    Empty,
    Active,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum AnimationState {
    #[default]
    Init,
    Stack(usize),
    Done,
}

#[derive(Component)]
struct EffectStackItem {
    state: AnimationState,
    index: usize,
}

fn spawn_effect_stack(
    mut commands: Commands,
    env: Res<Environment>,
    mut items: Local<Vec<Entity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    graphs: ResMut<Assets<AnimationGraph>>,
    mut next_state: ResMut<NextState<StackState>>,
) {
    next_state.set(if env.stack.is_empty() {
        StackState::Empty
    } else {
        StackState::Active
    });

    while items.len() < env.stack.len() {
        let index = items.len();

        let entity = commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(Plane3d::default().mesh().size(1.0, 1.0 / 48. * 20.)),
                    ..default()
                },
                EffectStackItem {
                    state: AnimationState::Init,
                    index,
                },
                AnimationPlayer::default(),
                Name::new("effect"),
                graphs.reserve_handle(),
            ))
            .id();
        commands.entity(entity).insert(AnimationTarget {
            id: AnimationTargetId::from_names([Name::new("effect")].iter()),
            player: entity,
        });
        items.push(entity);
    }
}

fn update_effect_stack(
    env: Res<Environment>,
    board: Res<Board>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(
        &mut EffectStackItem,
        &mut Transform,
        &mut Handle<StandardMaterial>,
    )>,
    camera: Query<&Transform, (With<Camera>, Without<EffectStackItem>)>,
    catalog: Res<Catalog>,
) {
    for (item, mut transform, mut material) in query.iter_mut() {
        if let Some(stack_item) = env.stack.as_slice().get(item.index) {
            if item.state == AnimationState::Init || item.state == AnimationState::Done {
                let zone = if let Ok(zone) = env.find_zone(stack_item.source) {
                    zone
                } else {
                    continue;
                };
                let card = env.find_card(stack_item.source).unwrap();
                let archetype = &catalog[card.archetype_id];
                let card_image = asset_server.load_with_cache(format!(
                    "cards/{}/image.main.png#stack",
                    archetype.safe_name
                ));
                *material = materials.add(StandardMaterial {
                    base_color_texture: Some(card_image),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                });

                let camera_pos = camera.single().translation;
                let base = Vec3::new(0.0, 3.0, 0.0);
                let mut target_transform =
                    Transform::from_translation(base).looking_at(camera_pos + base, Vec3::Y);
                target_transform.rotate_local_x(-std::f32::consts::PI / 2.0);
                target_transform.rotate_local_y(std::f32::consts::PI);

                let translation = board
                    .get_zone_transform(stack_item.source, zone, env.player, camera_pos)
                    .map(|transform| transform.translation);
                if let Some(translation) = translation {
                    target_transform.translation = translation;
                    target_transform.translation.y = 3.0;
                }
                target_transform.scale = Vec3::ZERO;
                *transform = target_transform;
            }
        }
    }
}

fn animate_effect_stack(
    mut commands: Commands,
    env: Res<Environment>,
    mut query: Query<(
        Entity,
        &mut EffectStackItem,
        &Transform,
        &mut AnimationPlayer,
        &mut Handle<AnimationGraph>,
    )>,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (entity, mut item, transform, mut anim, mut graph) in query.iter_mut() {
        let active = env.stack.len() > item.index;
        let state = if active {
            AnimationState::Stack(env.stack.len() - item.index - 1)
        } else {
            AnimationState::Done
        };

        if item.state == state {
            continue;
        }
        item.state = state;

        let mut animation = AnimationClip::default();

        if let AnimationState::Stack(index) = item.state {
            let keyframe_timestamps = vec![0.0, 0.1, 0.2];
            let target_transform = Transform::from_translation(Vec3::new(
                2.0 - 0.1 * index as f32,
                3.0 - 0.01 * index as f32,
                0.0,
            ));
            animation.add_curve_to_target(
                AnimationTargetId::from_names([Name::new("effect")].iter()),
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
                AnimationTargetId::from_names([Name::new("effect")].iter()),
                VariableCurve {
                    keyframe_timestamps: keyframe_timestamps.clone(),
                    keyframes: Keyframes::Scale(vec![
                        transform.scale,
                        Vec3::splat(1.0),
                        Vec3::splat(1.0),
                    ]),
                    interpolation: Interpolation::Linear,
                },
            );

            let (anim_graph, animation_index) =
                AnimationGraph::from_clip(animations.add(animation));
            *graph = graphs.add(anim_graph);
            *anim = AnimationPlayer::default();
            anim.play(animation_index);
            commands.add(RegisterAnimation::Async);
        } else if item.state == AnimationState::Done {
            let keyframe_timestamps = vec![0.0, 0.5, 0.6, 0.7];
            animation.add_curve_to_target(
                AnimationTargetId::from_names([Name::new("effect")].iter()),
                VariableCurve {
                    keyframe_timestamps: keyframe_timestamps.clone(),
                    keyframes: Keyframes::Translation(vec![
                        transform.translation,
                        transform.translation,
                        Vec3::new(
                            transform.translation.x + 10.0,
                            transform.translation.y,
                            transform.translation.z,
                        ),
                        Vec3::new(
                            transform.translation.x + 10.0,
                            transform.translation.y,
                            transform.translation.z,
                        ),
                    ]),
                    interpolation: Interpolation::Linear,
                },
            );

            let (anim_graph, animation_index) =
                AnimationGraph::from_clip(animations.add(animation));
            *graph = graphs.add(anim_graph);
            *anim = AnimationPlayer::default();
            anim.play(animation_index);
            commands.add(RegisterAnimation::Sync(entity));
        }
    }
}
