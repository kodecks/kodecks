use super::ui::UIEvent;
use crate::{
    assets::AssetServerExt,
    scene::{
        card::{Catalog, CARD_HEIGHT, CARD_WIDTH},
        game::board::Environment,
        GlobalState,
    },
};
use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use bevy_mod_picking::prelude::*;

pub struct DeckPlugin;

impl Plugin for DeckPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GraveyardHovered>()
            .add_systems(OnEnter(GlobalState::GameLoading), init)
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup)
            .add_systems(
                Update,
                (
                    (update_decks, update_graveyards)
                        .run_if(resource_exists_and_changed::<Environment>),
                    graveyard_hovered.run_if(on_event::<GraveyardHovered>()),
                )
                    .run_if(
                        in_state(GlobalState::GameMain).or_else(in_state(GlobalState::GameLoading)),
                    ),
            )
            .add_systems(
                OnEnter(GlobalState::GameLoading),
                (update_decks, update_graveyards),
            );
    }
}

#[derive(Component)]
enum Deck {
    Player,
    Opponent,
}

#[derive(Component)]
enum Graveyard {
    Player,
    Opponent,
}

#[derive(Event)]
struct GraveyardHovered(Graveyard);

fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let deck_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_cache("frames/back.png")),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let graveyard_material: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_cache("frames/graveyard.png")),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let mut deck_mesh = Cuboid::new(CARD_WIDTH, 1.0, CARD_HEIGHT).mesh().build();
    if let Some(VertexAttributeValues::Float32x2(v)) = deck_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
    {
        v[0] = [0.0, 1.0 - 1.0 / CARD_WIDTH];
        v[1] = [1.0, 1.0 - 1.0 / CARD_WIDTH];
        v[2] = [1.0, 1.0];
        v[3] = [0.0, 1.0];

        v[4] = [1.0, 0.0];
        v[5] = [0.0, 0.0];
        v[6] = [0.0, 1.0 / CARD_WIDTH];
        v[7] = [1.0, 1.0 / CARD_WIDTH];

        v[8] = [1.0 - 1.0 / CARD_WIDTH, 0.0];
        v[9] = [1.0, 0.0];
        v[10] = [1.0, 1.0];
        v[11] = [1.0 - 1.0 / CARD_WIDTH, 1.0];

        v[12] = [1.0 / CARD_WIDTH, 0.0];
        v[13] = [0.0, 0.0];
        v[14] = [0.0, 1.0];
        v[15] = [1.0 / CARD_WIDTH, 1.0];
    }

    let deck_mesh = meshes.add(deck_mesh);

    commands.spawn((
        PbrBundle {
            mesh: deck_mesh.clone(),
            material: deck_material.clone(),
            transform: Transform::from_translation(Vec3::new(5.0, 1.0, 3.0)).with_scale(Vec3::new(
                0.8 / CARD_WIDTH,
                1.0,
                0.8 / CARD_WIDTH,
            )),
            visibility: Visibility::Hidden,
            ..default()
        },
        Deck::Player,
    ));

    commands.spawn((
        PbrBundle {
            mesh: deck_mesh.clone(),
            material: deck_material.clone(),
            transform: Transform::from_translation(Vec3::new(5.0, 1.0, -1.5))
                .with_scale(Vec3::new(0.8 / CARD_WIDTH, 1.0, 0.8 / CARD_WIDTH))
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            visibility: Visibility::Hidden,
            ..default()
        },
        Deck::Opponent,
    ));

    commands.spawn((
        PbrBundle {
            mesh: deck_mesh.clone(),
            material: graveyard_material.clone(),
            transform: Transform::from_translation(Vec3::new(3.9, 1.0, 3.0)).with_scale(Vec3::new(
                0.8 / CARD_WIDTH,
                1.0,
                0.8 / CARD_WIDTH,
            )),
            visibility: Visibility::Hidden,
            ..default()
        },
        Graveyard::Player,
        On::<Pointer<Over>>::commands_mut(move |_, commands| {
            commands.add(move |w: &mut World| {
                w.send_event(GraveyardHovered(Graveyard::Player));
            });
        }),
    ));

    commands.spawn((
        PbrBundle {
            mesh: deck_mesh.clone(),
            material: graveyard_material,
            transform: Transform::from_translation(Vec3::new(3.9, 1.0, -1.5))
                .with_scale(Vec3::new(0.8 / CARD_WIDTH, 1.0, 0.8 / CARD_WIDTH))
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            visibility: Visibility::Hidden,
            ..default()
        },
        Graveyard::Opponent,
        On::<Pointer<Over>>::commands_mut(move |_, commands| {
            commands.add(move |w: &mut World| {
                w.send_event(GraveyardHovered(Graveyard::Opponent));
            });
        }),
    ));
}

type DeckQuery<'world, 'state> = Query<'world, 'state, Entity, Or<(With<Deck>, With<Graveyard>)>>;

fn cleanup(mut commands: Commands, query: DeckQuery) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_decks(env: Res<Environment>, mut query: Query<(&Deck, &mut Transform, &mut Visibility)>) {
    for (deck, mut transform, mut visibility) in query.iter_mut() {
        let len = match deck {
            Deck::Player => env.players.get(env.player).unwrap().deck,
            Deck::Opponent => {
                env.players
                    .get(env.next_id(env.player).unwrap())
                    .unwrap()
                    .deck
            }
        };
        let height = 0.005 * len as f32;
        transform.scale = Vec3::new(0.8 / CARD_WIDTH, height, 0.8 / CARD_WIDTH);
        *visibility = if len > 0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_graveyards(
    env: Res<Environment>,
    mut query: Query<(
        &Graveyard,
        &mut Transform,
        &mut Handle<StandardMaterial>,
        &mut Visibility,
    )>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    catalog: Res<Catalog>,
) {
    for (graveyard, mut transform, mut material, mut visibility) in query.iter_mut() {
        let player = match graveyard {
            Graveyard::Player => env.players.get(env.player).unwrap(),
            Graveyard::Opponent => env.players.next_player(env.player).unwrap(),
        };

        let len = player.graveyard.len() + 1;
        let height = 0.005 * len as f32;
        transform.scale = Vec3::new(0.8 / CARD_WIDTH, height, 0.8 / CARD_WIDTH);
        if let Some(last) = player.graveyard.last() {
            let archetype = &catalog[last.archetype_id];
            let card_image = asset_server
                .load_with_cache(format!("cards/{}/image.main.png", archetype.safe_name));
            *material = materials.add(StandardMaterial {
                base_color_texture: Some(card_image),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            *visibility = Visibility::Visible
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn graveyard_hovered(
    env: Res<Environment>,
    mut reader: EventReader<GraveyardHovered>,
    mut writer: EventWriter<UIEvent>,
) {
    for GraveyardHovered(player) in reader.read() {
        let player = match player {
            Graveyard::Player => env.players.get(env.player).unwrap(),
            Graveyard::Opponent => env.players.get(env.next_id(env.player).unwrap()).unwrap(),
        };
        if let Some(card) = player.graveyard.last() {
            writer.send(UIEvent::CardHovered(card.id));
        }
    }
}
