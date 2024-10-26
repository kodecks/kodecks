use crate::{
    assets::AssetServerExt,
    scene::{
        game::{board::Environment, event::ShardUpdated},
        GlobalState,
    },
};
use bevy::prelude::*;
use kodecks::color::Color;

pub struct ShardPlugin;

impl Plugin for ShardPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SpawningState>()
            .add_systems(Startup, setup)
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup)
            .add_systems(
                Update,
                (update.run_if(
                    on_event::<ShardUpdated>()
                        .or_else(in_state(SpawningState::Spawning))
                        .and_then(resource_exists::<ShardAssets>),
                ),)
                    .run_if(in_state(GlobalState::GameMain)),
            );
    }
}

#[derive(Resource)]
struct ShardAssets {
    mesh: Handle<Mesh>,
    red: Handle<StandardMaterial>,
    green: Handle<StandardMaterial>,
    blue: Handle<StandardMaterial>,
    yellow: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Plane3d::default().mesh().size(0.3, 0.3));

    let red = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_cache("shards/red.png")),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let green = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_cache("shards/green.png")),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let blue = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_cache("shards/blue.png")),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let yellow = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_cache("shards/yellow.png")),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let assets = ShardAssets {
        mesh,
        red,
        green,
        blue,
        yellow,
    };
    commands.insert_resource(assets);
}

#[derive(States, Clone, Default, Copy, Debug, PartialEq, Eq, Hash)]
enum SpawningState {
    #[default]
    Ready,
    Spawning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Player,
    Opponent,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct Shard {
    index: usize,
    side: Side,
    color: Color,
}

fn update(
    mut commands: Commands,
    env: Res<Environment>,
    assets: Res<ShardAssets>,
    mut query: Query<(
        &mut Shard,
        &mut Transform,
        &mut Handle<StandardMaterial>,
        &mut Visibility,
    )>,
    mut next_state: ResMut<NextState<SpawningState>>,
) {
    let mut shards = vec![];

    let player_shards = &env.players.get(env.player).shards;
    let opponent_shards = &env.players.next_player(env.player).shards;

    let mut player_count = 0;
    for color in Color::iter_all() {
        let count = player_shards.get(color) as usize;
        for _ in 0..count {
            shards.push(Shard {
                index: player_count,
                side: Side::Player,
                color,
            });
            player_count += 1;
        }
    }

    let mut opponent_count = 0;
    for color in Color::iter_all() {
        let count = opponent_shards.get(color) as usize;
        for _ in 0..count {
            shards.push(Shard {
                index: opponent_count,
                side: Side::Opponent,
                color,
            });
            opponent_count += 1;
        }
    }

    for (mut shard, mut transform, mut material, mut visibility) in query.iter_mut() {
        if let Some(item) = shards.pop() {
            if *shard != item {
                *shard = item;
                let position = Vec3::new(
                    match shard.side {
                        Side::Player => (item.index as f32 - (player_count - 1) as f32 * 0.5) * 0.4,
                        Side::Opponent => {
                            (item.index as f32 - (opponent_count - 1) as f32 * 0.5) * 0.4
                        }
                    },
                    0.2,
                    match shard.side {
                        Side::Player => 2.8,
                        Side::Opponent => -2.4,
                    },
                );
                *material = match shard.color {
                    Color::RED => assets.red.clone(),
                    Color::GREEN => assets.green.clone(),
                    Color::BLUE => assets.blue.clone(),
                    Color::YELLOW => assets.yellow.clone(),
                    _ => assets.red.clone(),
                };
                *transform = Transform::from_translation(position);
                *visibility = Visibility::Visible;
            }
        } else {
            *shard = Shard {
                index: 0,
                side: Side::Opponent,
                color: Color::COLORLESS,
            };
            *visibility = Visibility::Hidden;
        }
    }

    next_state.set(SpawningState::Ready);
    while shards.pop().is_some() {
        commands.spawn((
            PbrBundle {
                mesh: assets.mesh.clone(),
                material: assets.red.clone(),
                visibility: Visibility::Hidden,
                ..default()
            },
            Shard {
                index: 0,
                side: Side::Opponent,
                color: Color::COLORLESS,
            },
        ));

        next_state.set(SpawningState::Spawning);
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<Shard>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
