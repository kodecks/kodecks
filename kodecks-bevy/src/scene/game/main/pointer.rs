use crate::{assets::AssetServerExt, scene::GlobalState};

use super::dialog::DialogMessages;
use bevy::prelude::*;
use kodecks::message;

pub struct PointerPlugin;

impl Plugin for PointerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    update_pointers,
                    update.run_if(resource_changed_or_removed::<DialogMessages>()),
                )
                    .run_if(in_state(GlobalState::GameMain)),
            )
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup);
    }
}

#[derive(Resource)]
struct PointerAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Component, Deref)]
pub struct TargetMarker(message::PointerTarget);

impl TargetMarker {
    pub fn new(target: message::PointerTarget) -> Self {
        TargetMarker(target)
    }
}

#[derive(Component, Deref)]
struct Pointer(message::Pointer);

#[derive(Bundle)]
pub struct PointerBundle {
    pointer: Pointer,
    pbr: PbrBundle,
}

impl PointerBundle {
    fn new(pointer: message::Pointer) -> Self {
        PointerBundle {
            pointer: Pointer(pointer),
            pbr: PbrBundle::default(),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mesh = meshes.add(Plane3d::default().mesh().size(0.4, 0.4));
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_cache("pointers/arrow.png")),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    });

    commands.insert_resource(PointerAssets { mesh, material });
}

fn update(
    mut commands: Commands,
    messages: Option<Res<DialogMessages>>,
    query: Query<Entity, With<Pointer>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    if let Some(messages) = messages {
        if let Some(last) = messages.messages.last() {
            for pointer in &last.pointers {
                commands.spawn(PointerBundle::new(*pointer));
            }
        }
    }
}

type PointerQuery<'world, 'state, 'a> = Query<
    'world,
    'state,
    (
        &'a Pointer,
        &'a mut Transform,
        &'a mut Handle<Mesh>,
        &'a mut Handle<StandardMaterial>,
    ),
    Added<Pointer>,
>;

fn update_pointers(
    assets: Res<PointerAssets>,
    mut query: PointerQuery,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    marker_query: Query<(&TargetMarker, &Transform), Without<Pointer>>,
) {
    let (camera, camera_transform) = camera_query.single();

    for (pointer, mut transform, mut mesh, mut material) in query.iter_mut() {
        *mesh = assets.mesh.clone();
        *material = assets.material.clone();
        if let Some((_, target_transform)) = marker_query
            .iter()
            .find(|(marker, _)| marker.0 == pointer.target)
        {
            let y = camera
                .world_to_ndc(camera_transform, target_transform.translation)
                .map(|vec| vec.y)
                .unwrap_or(-1.0);

            let mut target_transform = *target_transform;
            target_transform.translation.y += 0.5;
            if y > 0.0 {
                target_transform.translation.z += 0.5;
                target_transform.rotation = Quat::from_rotation_y(-std::f32::consts::PI);
            } else {
                target_transform.translation.z -= 0.5;
                target_transform.rotation = Quat::IDENTITY;
            }
            *transform = target_transform;
        }
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<Pointer>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
