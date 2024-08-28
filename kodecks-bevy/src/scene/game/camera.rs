use crate::scene::GlobalState;
use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    pbr::ShadowFilteringMethod,
    prelude::*,
};

const CAMERA_POSITION: Vec3 = Vec3::new(0.0, 11.0, 1.0);
const CAMERA_TARGET: Vec3 = Vec3::new(0.0, 0.0, 0.8);

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init)
            .add_systems(OnEnter(GlobalState::GameLoading), setup)
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup);
    }
}

fn init(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_translation(CAMERA_POSITION)
                .looking_at(CAMERA_TARGET, Vec3::Y),
            ..default()
        },
        ShadowFilteringMethod::Hardware2x2,
        BloomSettings::NATURAL,
        AnimationPlayer::default(),
        Name::new("camera"),
    ));
}

fn setup(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::srgb(1.0, 1.0, 1.0),
            intensity: 1_000_000.,
            range: 40.0,
            radius: 5.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 12.0, 0.0),
        ..default()
    });
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<PointLight>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
