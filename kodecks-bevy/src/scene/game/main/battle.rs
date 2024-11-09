use crate::{
    assets::AssetServerExt,
    scene::{
        game::board::{Board, Environment},
        GlobalState,
    },
};
use bevy::prelude::*;
use kodecks::zone::Zone;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup)
            .add_systems(
                Update,
                (update_battle.run_if(resource_exists_and_changed::<Board>))
                    .run_if(in_state(GlobalState::GameMain)),
            );
    }
}

#[derive(Component)]
struct Arrow;

#[derive(Resource)]
struct BattleAssets {
    meshes: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_cache("pointers/attack.png")),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.insert_resource(BattleAssets {
        meshes: meshes.add(Plane3d::default().mesh().size(0.3, 0.3)),
        material,
    });
}

fn update_battle(
    mut commands: Commands,
    assets: Res<BattleAssets>,
    env: Res<Environment>,
    board: Res<Board>,
    arrows: Query<Entity, With<Arrow>>,
    camera: Query<&Transform, With<Camera>>,
) {
    for entity in arrows.iter() {
        commands.entity(entity).despawn();
    }

    let camera_pos = camera.single().translation;
    for (attacker, blocker) in board.blocking_pairs() {
        let attacker_zone = if let Ok(zone) = env.find_zone(attacker.id) {
            zone
        } else {
            continue;
        };
        let defender_zone = if let Ok(zone) = env.find_zone(blocker.id) {
            zone
        } else {
            continue;
        };
        if attacker_zone.zone == Zone::Field && defender_zone.zone == Zone::Field {
            let mut attacker_pos = board
                .get_zone_transform(attacker.id, attacker_zone, env.player, camera_pos)
                .map(|transform| transform.translation)
                .unwrap_or_default();
            attacker_pos.y = 0.5;

            let mut defender_pos = board
                .get_zone_transform(blocker.id, defender_zone, env.player, camera_pos)
                .map(|transform| transform.translation)
                .unwrap_or_default();
            defender_pos.y = 0.5;

            let points = [attacker_pos, defender_pos];
            let b_spline = CubicCardinalSpline::new(0.6, points).to_curve();

            let points = [b_spline.position(0.2), b_spline.position(0.8)];
            let b_spline: CubicCurve<Vec3> = CubicCardinalSpline::new(0.6, points).to_curve();

            let arrows = ((points[0] - points[1]).length() * 4.0).floor() as usize;

            for (pos, vel) in b_spline
                .iter_positions(arrows)
                .zip(b_spline.iter_velocities(arrows))
            {
                let transform = Transform::from_translation(pos).looking_to(vel, Vec3::Y);
                commands.spawn((
                    PbrBundle {
                        mesh: assets.meshes.clone(),
                        material: assets.material.clone(),
                        transform,
                        ..default()
                    },
                    Arrow,
                ));
            }
        }
    }
}

fn cleanup(mut commands: Commands, arrows: Query<Entity, With<Arrow>>) {
    for entity in arrows.iter() {
        commands.entity(entity).despawn();
    }
}
