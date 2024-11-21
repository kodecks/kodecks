use crate::{
    painter::numbers::{Alignment, DrawOptions, NumberPainter},
    scene::{
        game::{board::Environment, event::LifeUpdated},
        GlobalState,
    },
};
use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use image::{DynamicImage, RgbaImage};
use kodecks::message::PointerTarget;

use super::pointer::TargetMarker;

pub struct LifePlugin;

impl Plugin for LifePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalState::GameInit), init)
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup)
            .add_systems(
                Update,
                (update.run_if(on_event::<LifeUpdated>()), update_damage).run_if(
                    in_state(GlobalState::GameMain).or_else(in_state(GlobalState::GameResult)),
                ),
            )
            .add_systems(OnEnter(GlobalState::GameLoading), update);
    }
}

#[derive(Component, PartialEq, Eq, Clone, Copy)]
enum LifeOverlay {
    Player,
    Opponent,
}

#[derive(Component, PartialEq, Clone, Copy)]
struct DamageOverlay {
    z: f32,
}

fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(2.4, 0.6)),
            material: material.clone(),
            transform: Transform::from_translation(Vec3::new(0., 0.2, 2.2)),
            ..default()
        },
        LifeOverlay::Player,
        TargetMarker::new(PointerTarget::PlayersLife),
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(2.4, 0.6)),
            material: material.clone(),
            transform: Transform::from_translation(Vec3::new(0., 0.2, -1.8)),
            ..default()
        },
        LifeOverlay::Opponent,
        TargetMarker::new(PointerTarget::OpponentsLife),
    ));
}

type OverlayQuery<'world, 'state> =
    Query<'world, 'state, Entity, Or<(With<LifeOverlay>, With<DamageOverlay>)>>;

fn cleanup(mut commands: Commands, query: OverlayQuery) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn update(
    env: Res<Environment>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&LifeOverlay, &mut Handle<StandardMaterial>, &Transform)>,
    mut images: ResMut<Assets<Image>>,
    mut event_reader: EventReader<LifeUpdated>,
) {
    let player_life = env.players.get(env.player).unwrap().stats.life;
    let opponent_life = env.players.next_player(env.player).unwrap().stats.life;

    let ((_, mut material, transform), life, delta) = match event_reader.read().next() {
        Some(LifeUpdated { player, delta }) => {
            if *player == env.player {
                (
                    query
                        .iter_mut()
                        .find(|(overlay, _, _)| **overlay == LifeOverlay::Player)
                        .unwrap(),
                    player_life,
                    delta,
                )
            } else {
                (
                    query
                        .iter_mut()
                        .find(|(overlay, _, _)| **overlay == LifeOverlay::Opponent)
                        .unwrap(),
                    opponent_life,
                    delta,
                )
            }
        }
        _ => return,
    };

    let mut id_overlay = DynamicImage::ImageRgba8(RgbaImage::new(48, 12));
    NumberPainter::default().draw(
        &life.to_string(),
        &DrawOptions {
            x: id_overlay.width() / 2,
            y: id_overlay.height() / 2,
            h_align: Alignment::Center,
            v_align: Alignment::Center,
            background: [255, 255, 255, 255].into(),
            foreground: [0, 0, 0, 255].into(),
        },
        &mut id_overlay,
    );
    let id_texture = images.add(Image::new_fill(
        Extent3d {
            width: id_overlay.width(),
            height: id_overlay.height(),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        id_overlay.as_bytes(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ));

    *material = materials.add(StandardMaterial {
        base_color_texture: Some(id_texture),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    if *delta != 0 {
        let mut delta_overlay = DynamicImage::ImageRgba8(RgbaImage::new(48, 12));
        NumberPainter::default().draw(
            &delta.to_string(),
            &DrawOptions {
                x: delta_overlay.width() / 2,
                y: delta_overlay.height() / 2,
                h_align: Alignment::Center,
                v_align: Alignment::Center,
                background: [255, 0, 0, 255].into(),
                foreground: [0, 0, 0, 255].into(),
            },
            &mut delta_overlay,
        );
        let id_texture = images.add(Image::new_fill(
            Extent3d {
                width: delta_overlay.width(),
                height: delta_overlay.height(),
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            delta_overlay.as_bytes(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        ));

        let mut transform = *transform;
        transform.translation.y = 0.5;
        transform.translation.z -= 0.3;
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Plane3d::default().mesh().size(1.6, 0.4)),
                material: materials.add(StandardMaterial {
                    base_color_texture: Some(id_texture),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                }),
                transform,
                ..default()
            },
            DamageOverlay {
                z: transform.translation.z,
            },
        ));
    }
}

fn update_damage(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &DamageOverlay)>,
    time: Res<Time>,
) {
    for (entity, mut transform, overlay) in query.iter_mut() {
        transform.translation.z -= time.delta_seconds() * 0.5;
        if transform.translation.z < overlay.z - 0.5 {
            commands.entity(entity).despawn();
        }
    }
}
