use super::engine::PlayerEvent;
use super::ui::UIEvent;
use crate::assets::AssetHandleStore;
use crate::painter::frames::CardFramePainter;
use crate::painter::numbers::{Alignment, DrawOptions, NumberPainter};
use crate::scene::game::board::{self, AvailableActionList, Board, Environment};
use crate::scene::GlobalState;
use ability::AbilityOverlay;
use bevy::animation::{AnimationTarget, AnimationTargetId};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::system::SystemParam;
use bevy::math::bounding::{Aabb3d, BoundingVolume, IntersectsVolume, RayCast3d};
use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy::utils::{HashMap, HashSet};
use bevy_mod_picking::prelude::*;
use frame::CardFrame;
use image::{DynamicImage, RgbaImage};
use kodecks::card::{ArchetypeId, CardArchetype, CardSnapshot};
use kodecks::id::ObjectId;
use kodecks::zone::Zone;
use kodecks_catalog::CATALOG;
use number::{NumberOverlay, NumberOverlayKey};
use std::cmp::Ordering;
use std::f32::consts::PI;
use web_time::Instant;

pub const CARD_WIDTH: f32 = 36.0;
pub const CARD_HEIGHT: f32 = 48.0;

mod ability;
mod frame;
mod number;

pub struct CardPlugin;

impl Plugin for CardPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<CardState>()
            .init_resource::<CardFramePainter>()
            .add_event::<CardDrag>()
            .add_systems(Startup, (frame::setup, ability::setup))
            .add_systems(
                Update,
                (
                    ((
                        update_cards,
                        (
                            frame::update_frame_overlay,
                            number::update_number_overlay,
                            ability::update_ability_overlay,
                        ),
                    )
                        .chain())
                    .run_if(
                        resource_exists_and_changed::<Board>
                            .or_else(resource_exists_and_changed::<Environment>),
                    ),
                    (initialize_card_bundles, update_card, update_state)
                        .chain()
                        .run_if(resource_exists_and_changed::<Environment>),
                    handle_card_events.run_if(
                        on_event::<Pointer<Click>>()
                            .or_else(on_event::<Pointer<Drag>>())
                            .or_else(on_event::<Pointer<DragEnd>>()),
                    ),
                )
                    .run_if(in_state(GlobalState::GameMain)),
            )
            .add_systems(OnEnter(GlobalState::GameLoading), init)
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup);
    }
}

fn init(mut commands: Commands, mut next_state: ResMut<NextState<CardState>>) {
    next_state.set(CardState::default());
    commands.insert_resource(CardAssets::default());
    commands.insert_resource(AssetHandleStore::<NumberOverlayKey, StandardMaterial>::default());
}

fn cleanup(
    mut commands: Commands,
    query: Query<Entity, With<Card>>,
    mut next_state: ResMut<NextState<CardState>>,
) {
    next_state.set(CardState::default());
    commands.remove_resource::<CardAssets>();
    commands.remove_resource::<AssetHandleStore<NumberOverlayKey, StandardMaterial>>();
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum CardState {
    #[default]
    Ready,
    Loaded,
    Animating,
}

#[derive(Component, Deref, DerefMut)]
pub struct Card(CardSnapshot);

impl From<CardSnapshot> for Card {
    fn from(card: CardSnapshot) -> Self {
        Self(card)
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct CardId(ObjectId);

impl From<ObjectId> for CardId {
    fn from(id: ObjectId) -> Self {
        Self(id)
    }
}

#[derive(Resource, Default)]
struct CardAssets {
    pub cards: HashMap<ObjectId, CardAssetEntry>,
    pub loading_cards: HashSet<ObjectId>,
}

struct CardAssetEntry {
    entity: Entity,
    archetype_id: Option<ArchetypeId>,
    time: Instant,
}

fn update_state(state: Res<State<CardState>>, mut next_state: ResMut<NextState<CardState>>) {
    if *state == CardState::Ready {
        next_state.set(CardState::Loaded);
    }
}

fn update_card(
    env: Res<board::Environment>,
    changed_query: Query<(&Card, Entity), Changed<Card>>,
    children: Query<&mut Children>,
    mut card_frame_query: Query<(&mut Handle<StandardMaterial>, &CardImage)>,
    mut transform_query: Query<&mut Transform>,
    mut builder: CardBundleBuilder,
) {
    for (card, entity) in changed_query.iter() {
        for child in children.iter_descendants(entity) {
            let archetype = &CATALOG[card.archetype_id];
            if let Ok((mut material, frame)) = card_frame_query.get_mut(child) {
                *material = builder.load_image(*frame, archetype);
            }
        }

        if env.find_zone(card.id).map(|zone| zone.zone) != Ok(Zone::Graveyard) {
            if let Ok(mut transform) = transform_query.get_mut(entity) {
                if transform.translation.y < 0.0 {
                    let z = if card.owner == env.player { 2.5 } else { -2.5 };
                    *transform = Transform::from_rotation(Quat::from_rotation_z(PI))
                        .with_translation(Vec3::new(5.0, 2.0, z))
                        .with_scale(Vec3::splat(0.8));
                }
            }
        }
    }
}

fn initialize_card_bundles(
    env: Res<board::Environment>,
    mut assets: ResMut<CardAssets>,
    mut commands: Commands,
    mut query: Query<&mut Card>,
    mut builder: CardBundleBuilder,
) {
    let cards = env.players.iter().flat_map(|player| {
        player
            .hand
            .iter()
            .map(|hand| &hand.card)
            .chain(player.field.iter().map(|item| &item.card))
            .chain(player.graveyard.last())
    });
    for card in cards {
        let entry = assets.cards.entry(card.id);
        let entry = entry.or_insert_with(|| {
            let opponent = env.find_zone(card.id).unwrap().player != env.player;
            let entity = builder.spawn(card.clone(), opponent);
            CardAssetEntry {
                entity,
                archetype_id: None,
                time: Instant::now(),
            }
        });
        entry.time = Instant::now();
        if Some(card.archetype_id) != entry.archetype_id {
            entry.archetype_id = Some(card.archetype_id);
            if let Ok(mut old) = query.get_mut(entry.entity) {
                **old = card.clone();
            }
            assets.loading_cards.insert(card.id);
        }
    }

    let old_cards = assets
        .cards
        .iter()
        .filter(|(_, entry)| entry.time.elapsed().as_secs() > 10)
        .map(|(id, entry)| {
            commands.add(DespawnRecursive {
                entity: entry.entity,
            });
            *id
        })
        .collect::<Vec<_>>();

    for card in old_cards {
        assets.cards.remove(&card);
    }
}

fn update_cards(env: Res<board::Environment>, mut query: Query<&mut Card>) {
    query.iter_mut().for_each(|mut card| {
        if let Ok(new) = env.find_card(card.id) {
            *card = new.clone().into();
        }
    });
}

#[derive(Event)]
pub struct CardDrag {
    entity: Entity,
    position: Vec2,
    drop: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardImage {
    Frame,
    Image,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardSize {
    Large,
    Small,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct CardIdOverlay;

struct SharedHandles {
    frame_mesh: Handle<Mesh>,
    shadow_mesh: Handle<Mesh>,
    frame_small_mesh: Handle<Mesh>,
    image_mesh: Handle<Mesh>,
    id_overlay_mesh: Handle<Mesh>,
    ability_overlay_mesh: Handle<Mesh>,
    card_back_material: Handle<StandardMaterial>,
    shadow_material: Handle<StandardMaterial>,
}

#[derive(Bundle)]
pub struct CardBundle {
    spatial: SpatialBundle,
    card: Card,
    pick: PickableBundle,
    hover: On<Pointer<Over>>,
    animation: AnimationPlayer,
    animation_name: Name,
    graph: Handle<AnimationGraph>,
}

#[derive(SystemParam)]
pub struct CardBundleBuilder<'w, 's> {
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    images: ResMut<'w, Assets<Image>>,
    graphs: ResMut<'w, Assets<AnimationGraph>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    asset_server: Res<'w, AssetServer>,
    handles: Local<'s, Option<SharedHandles>>,
    id_store: Res<'w, AssetHandleStore<NumberOverlayKey, StandardMaterial>>,
}

impl<'w, 's> CardBundleBuilder<'w, 's> {
    pub fn spawn(&mut self, card: CardSnapshot, opponent: bool) -> Entity {
        let card_id = card.id;
        let archetype = &CATALOG[card.archetype_id];

        let image = self.load_image(CardImage::Image, archetype);
        let frame = self.load_image(CardImage::Frame, archetype);
        let number_cost = self.number(&NumberOverlay::Cost, &card);
        let number_field = self.number(&NumberOverlay::Field, &card);

        let handles = self.handles.get_or_insert_with(|| SharedHandles {
            frame_mesh: self.meshes.reserve_handle(),
            shadow_mesh: self.meshes.reserve_handle(),
            frame_small_mesh: self.meshes.reserve_handle(),
            image_mesh: self.meshes.reserve_handle(),
            id_overlay_mesh: self.meshes.reserve_handle(),
            ability_overlay_mesh: self.meshes.reserve_handle(),
            card_back_material: self.materials.reserve_handle(),
            shadow_material: self.materials.reserve_handle(),
        });

        self.meshes.get_or_insert_with(handles.frame_mesh.id(), || {
            Plane3d::default()
                .mesh()
                .size(CARD_WIDTH, CARD_HEIGHT)
                .build()
        });

        self.meshes
            .get_or_insert_with(handles.shadow_mesh.id(), || {
                Plane3d::default().mesh().size(40.0, 80.0).build()
            });

        self.meshes
            .get_or_insert_with(handles.frame_small_mesh.id(), || {
                Plane3d::default().mesh().size(50.0, 50.0).build()
            });

        self.meshes.get_or_insert_with(handles.image_mesh.id(), || {
            Plane3d::default().mesh().size(32.0, 32.0).build()
        });

        self.meshes
            .get_or_insert_with(handles.id_overlay_mesh.id(), || {
                Plane3d::default().mesh().size(32.0, 32.0).build()
            });

        self.meshes
            .get_or_insert_with(handles.ability_overlay_mesh.id(), || {
                Plane3d::default().mesh().size(12.0, 12.0).build()
            });

        self.materials
            .get_or_insert_with(handles.card_back_material.id(), || {
                let texture_handle = self.asset_server.load("frames/back.png");
                StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    alpha_mode: bevy::prelude::AlphaMode::Blend,
                    ..default()
                }
            });

        self.materials
            .get_or_insert_with(handles.shadow_material.id(), || {
                let texture_handle = self.asset_server.load("frames/frame.png");
                StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    alpha_mode: bevy::prelude::AlphaMode::Multiply,
                    double_sided: true,
                    cull_mode: None,
                    emissive: LinearRgba::rgb(-100.0, -100.0, -100.0),
                    ..default()
                }
            });

        let mut id_overlay = DynamicImage::ImageRgba8(RgbaImage::new(32, 32));
        NumberPainter::default().draw(
            &card.id.to_string(),
            &DrawOptions {
                x: id_overlay.width() / 2,
                y: id_overlay.height() / 2,
                h_align: Alignment::Center,
                v_align: Alignment::Center,
                background: [255, 255, 255, 255].into(),
                foreground: [0, 0, 255, 255].into(),
            },
            &mut id_overlay,
        );
        let id_texture = self.images.add(Image::new_fill(
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

        let id_material = self.materials.add(StandardMaterial {
            base_color_texture: Some(id_texture),
            alpha_mode: bevy::prelude::AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        let card_entity = self
            .commands
            .spawn(CardBundle {
                spatial: SpatialBundle {
                    transform: Transform::from_xyz(0.0, -2.0, 0.0),
                    ..Default::default()
                },
                card: card.into(),
                pick: PickableBundle::default(),
                hover: On::<Pointer<Over>>::commands_mut(move |_, commands| {
                    commands.add(move |w: &mut World| {
                        w.send_event(UIEvent::CardHovered(card_id));
                    });
                }),
                animation: AnimationPlayer::default(),
                animation_name: Name::new("card"),
                graph: self.graphs.reserve_handle(),
            })
            .id();

        self.commands
            .entity(card_entity)
            .insert(AnimationTarget {
                id: AnimationTargetId::from_name(&Name::new("card")),
                player: card_entity,
            })
            .with_children(|parent| {
                parent.spawn((
                    PbrBundle {
                        mesh: handles.image_mesh.clone(),
                        transform: Transform::from_scale(Vec3::splat(1.0 / CARD_WIDTH))
                            .with_translation(Vec3::new(0.0, 0.001, 0.0))
                            .with_rotation(if opponent {
                                Quat::from_rotation_y(std::f32::consts::PI)
                            } else {
                                Quat::IDENTITY
                            }),
                        material: image,
                        ..default()
                    },
                    CardImage::Image,
                    CardSize::Small,
                    Name::new("card_image"),
                ));

                parent.spawn((
                    PbrBundle {
                        mesh: handles.frame_mesh.clone(),
                        transform: Transform::from_scale(Vec3::splat(1.0 / CARD_WIDTH))
                            .with_translation(Vec3::new(0.0, 0.0011, -1.0 / CARD_WIDTH * 6.0)),
                        material: frame,
                        ..default()
                    },
                    CardImage::Frame,
                    Name::new("card_frame"),
                    CardSize::Large,
                    AnimationTarget {
                        id: AnimationTargetId::from_names(
                            [Name::new("card"), Name::new("card_frame")].iter(),
                        ),
                        player: card_entity,
                    },
                ));
                parent.spawn((
                    PbrBundle {
                        mesh: handles.shadow_mesh.clone(),
                        material: handles.shadow_material.clone(),
                        transform: Transform::from_scale(Vec3::splat(1.0 / CARD_WIDTH))
                            .with_translation(Vec3::new(0.0, 0.0, -1.0 / CARD_WIDTH * 6.0)),
                        ..default()
                    },
                    Name::new("card_shadow"),
                    CardFrame::Shadow,
                    CardSize::Large,
                    AnimationTarget {
                        id: AnimationTargetId::from_names(
                            [Name::new("card"), Name::new("card_shadow")].iter(),
                        ),
                        player: card_entity,
                    },
                ));
                parent.spawn((
                    PbrBundle {
                        mesh: handles.frame_small_mesh.clone(),
                        transform: Transform::from_scale(Vec3::splat(0.0))
                            .with_translation(Vec3::new(0.0, 0.004, 0.0)),
                        ..default()
                    },
                    Name::new("card_small_overlay"),
                    CardFrame::SmallOverlay,
                    CardSize::Small,
                    AnimationTarget {
                        id: AnimationTargetId::from_names(
                            [Name::new("card"), Name::new("card_small_overlay")].iter(),
                        ),
                        player: card_entity,
                    },
                ));
                parent.spawn((
                    PbrBundle {
                        mesh: handles.frame_mesh.clone(),
                        material: number_cost,
                        transform: Transform::from_scale(Vec3::splat(1.0 / CARD_WIDTH))
                            .with_translation(Vec3::new(0.0, 0.01, -1.0 / CARD_WIDTH * 6.0)),
                        ..default()
                    },
                    Name::new("card_cost_overlay"),
                    NumberOverlay::Cost,
                ));
                parent.spawn((
                    PbrBundle {
                        mesh: handles.frame_small_mesh.clone(),
                        material: number_field,
                        transform: Transform::from_scale(Vec3::splat(0.0))
                            .with_translation(Vec3::new(0.0, 0.1, 0.0)),
                        ..default()
                    },
                    Name::new("card_field_overlay"),
                    NumberOverlay::Field,
                    CardSize::Small,
                    AnimationTarget {
                        id: AnimationTargetId::from_names(
                            [Name::new("card"), Name::new("card_field_overlay")].iter(),
                        ),
                        player: card_entity,
                    },
                ));
                parent.spawn((
                    PbrBundle {
                        mesh: handles.ability_overlay_mesh.clone(),
                        transform: Transform::from_scale(Vec3::splat(1.0 / CARD_WIDTH))
                            .with_translation(Vec3::new(
                                1.0 / CARD_WIDTH * 20.0,
                                0.1,
                                -1.0 / CARD_WIDTH * 7.0,
                            )),
                        material: self.materials.add(StandardMaterial {
                            base_color_texture: Some(self.asset_server.load("abilities/toxic.png")),
                            alpha_mode: bevy::prelude::AlphaMode::Blend,
                            ..default()
                        }),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    AbilityOverlay {
                        index: 0,
                        ability: None,
                    },
                ));
                parent.spawn((
                    PbrBundle {
                        mesh: handles.ability_overlay_mesh.clone(),
                        transform: Transform::from_scale(Vec3::splat(1.0 / CARD_WIDTH))
                            .with_translation(Vec3::new(
                                1.0 / CARD_WIDTH * 20.0,
                                0.1,
                                1.0 / CARD_WIDTH * 7.0,
                            )),
                        material: self.materials.add(StandardMaterial {
                            base_color_texture: Some(self.asset_server.load("abilities/toxic.png")),
                            alpha_mode: bevy::prelude::AlphaMode::Blend,
                            ..default()
                        }),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    AbilityOverlay {
                        index: 1,
                        ability: None,
                    },
                ));
                parent.spawn(PbrBundle {
                    mesh: handles.frame_mesh.clone(),
                    material: handles.card_back_material.clone(),
                    transform: Transform::from_scale(Vec3::splat(1.0 / CARD_WIDTH))
                        .with_rotation(Quat::from_rotation_z(std::f32::consts::PI))
                        .with_translation(Vec3::new(0.0, -0.001, 0.0)),
                    ..default()
                });
                parent.spawn((
                    PbrBundle {
                        mesh: handles.id_overlay_mesh.clone(),
                        material: id_material,
                        transform: Transform::from_scale(Vec3::splat(1.0 / 32.0 / 2.0))
                            .with_translation(Vec3::new(0.4, 0.2, 0.4)),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    CardIdOverlay,
                ));
            })
            .id()
    }

    pub fn load_image(
        &mut self,
        image: CardImage,
        archetype: &CardArchetype,
    ) -> Handle<StandardMaterial> {
        if archetype.id.is_empty() {
            return self.materials.add(StandardMaterial {
                base_color_texture: Some(self.asset_server.load("frames/back.png")),
                alpha_mode: bevy::prelude::AlphaMode::Blend,
                ..default()
            });
        }

        let image = match image {
            CardImage::Frame => self
                .asset_server
                .load(format!("cards/{}/image.main.png#hand", archetype.safe_name)),
            CardImage::Image => self.asset_server.load(format!(
                "cards/{}/image.main.png#image",
                archetype.safe_name
            )),
        };

        self.materials.add(StandardMaterial {
            base_color_texture: Some(image),
            alpha_mode: bevy::prelude::AlphaMode::Blend,
            unlit: true,
            ..default()
        })
    }

    pub fn number(
        &mut self,
        overlay: &NumberOverlay,
        card: &CardSnapshot,
    ) -> Handle<StandardMaterial> {
        let painter = CardFramePainter::default();
        let background_color = painter.get_color(Default::default());
        let foreground_color = painter.get_color(card.color());
        if *overlay == NumberOverlay::Cost {
            let cost = card.cost().value();

            let material = self
                .id_store
                .get(NumberOverlayKey::Cost { cost }, &self.materials);
            self.materials.get_or_insert_with(material.id(), || {
                let mut cost_overlay =
                    DynamicImage::ImageRgba8(RgbaImage::new(CARD_WIDTH as u32, CARD_HEIGHT as u32));
                NumberPainter::default().draw(
                    &cost.to_string(),
                    &DrawOptions {
                        x: 2,
                        y: 2,
                        h_align: Alignment::Start,
                        v_align: Alignment::Start,
                        background: [255, 255, 255, 255].into(),
                        foreground: background_color,
                    },
                    &mut cost_overlay,
                );
                let cost_texture = self.images.add(Image::new_fill(
                    Extent3d {
                        width: cost_overlay.width(),
                        height: cost_overlay.height(),
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    cost_overlay.as_bytes(),
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::RENDER_WORLD,
                ));

                StandardMaterial {
                    base_color_texture: Some(cost_texture),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                }
            });
            material
        } else {
            let material = self.id_store.get(
                NumberOverlayKey::Field {
                    color: foreground_color,
                    power: card.power().map(|p| p.value()),
                    diff: card.power().map(|p| p.diff()).unwrap_or(Ordering::Equal),
                },
                &self.materials,
            );

            self.materials.get_or_insert_with(material.id(), || {
                let mut overlay =
                    DynamicImage::ImageRgba8(RgbaImage::new(CARD_WIDTH as u32, CARD_WIDTH as u32));
                if let Some(power) = card.power() {
                    let foreground = match power.diff() {
                        Ordering::Less => [100, 100, 255, 255],
                        Ordering::Greater => [255, 100, 100, 255],
                        Ordering::Equal => [255, 255, 255, 255],
                    };
                    let power = power.value();
                    NumberPainter::default().draw(
                        &format!("{}", power).replace('0', "o"),
                        &DrawOptions {
                            x: 3,
                            y: 18,
                            h_align: Alignment::Start,
                            v_align: Alignment::Center,
                            background: background_color,
                            foreground: foreground.into(),
                        },
                        &mut overlay,
                    );
                }
                let overlay_texture = self.images.add(Image::new_fill(
                    Extent3d {
                        width: overlay.width(),
                        height: overlay.height(),
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    overlay.as_bytes(),
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::RENDER_WORLD,
                ));

                StandardMaterial {
                    base_color_texture: Some(overlay_texture),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                }
            });

            material
        }
    }
}

#[derive(SystemParam)]
pub struct CardEvents<'w, 's> {
    click: EventReader<'w, 's, Pointer<Click>>,
    drag: EventReader<'w, 's, Pointer<Drag>>,
    drag_end: EventReader<'w, 's, Pointer<DragEnd>>,
    player: EventWriter<'w, PlayerEvent>,
}

pub fn handle_card_events(
    board: Res<Board>,
    list: Res<AvailableActionList>,
    mut events: CardEvents,
    mut card_query: Query<(&mut Transform, &mut Card)>,
    parent_query: Query<&Parent>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    for event in events.click.read() {
        let mut target = event.target;
        while let Ok(parent) = parent_query.get(target) {
            target = parent.get();
            if let Ok((_, card)) = card_query.get(target) {
                events.player.send(PlayerEvent::CardClicked(card.id));
                break;
            }
        }
    }

    let mut drag = None;
    for event in events.drag.read() {
        let mut target = event.target;
        while let Ok(parent) = parent_query.get(target) {
            target = parent.get();
            if card_query.get(target).is_ok() {
                drag = Some(CardDrag {
                    entity: target,
                    position: event.pointer_location.position,
                    drop: false,
                });
                break;
            }
        }
    }
    for event in events.drag_end.read() {
        let mut target = event.target;
        while let Ok(parent) = parent_query.get(target) {
            target = parent.get();
            if card_query.get(target).is_ok() {
                drag = Some(CardDrag {
                    entity: target,
                    position: event.pointer_location.position,
                    drop: true,
                });
                break;
            }
        }
    }

    if let Some(drag) = drag {
        if let Ok((_, card)) = card_query.get(drag.entity) {
            if !board.player_hand.contains(&card.id) && !list.blockers().contains(&card.id) {
                return;
            }
        }

        let (camera, camera_transform) = camera_query.single();
        let top_left = Vec3::new(-0.5, -1.0, -0.5);
        let bottom_right = Vec3::new(0.5, 1.0, 0.5);

        let mut targets = if let Ok((transform, dragged_card)) = card_query.get(drag.entity) {
            let dragged_aabb = Aabb3d::from_point_cloud(
                transform.translation,
                transform.rotation,
                [top_left, bottom_right].iter().copied(),
            );

            card_query
                .iter()
                .filter(|(_, card)| card.id != dragged_card.id)
                .filter_map(|(transform, card)| {
                    let aabb = Aabb3d::from_point_cloud(
                        transform.translation,
                        transform.rotation,
                        [top_left, bottom_right].iter().copied(),
                    );
                    if aabb.intersects(&dragged_aabb) {
                        Some((card.id, (aabb.center() - dragged_aabb.center()).length()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        targets.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

        if let Ok((mut transform, card)) = card_query.get_mut(drag.entity) {
            let z = 3.5;
            let y = 2.5;

            let ray = camera.viewport_to_world(camera_transform, drag.position);
            let point = ray.and_then(|ray| {
                ray.intersect_plane(
                    Vec3::new(0.0, y, z),
                    InfinitePlane3d::new(Vec3::new(0.0, 10.0 - y, 3.0)),
                )
                .map(|distance| ray.get_point(distance))
            });

            let aabb = Aabb3d::new(Vec3::new(0.0, 1.0, 0.0), Vec3::new(10.0, 0.0, 2.0));
            let field_pos = ray.and_then(|ray| {
                RayCast3d::from_ray(ray, 100.0)
                    .aabb_intersection_at(&aabb)
                    .map(|distance| ray.get_point(distance))
            });

            if drag.drop && field_pos.is_some() {
                if let Some((target, _)) = targets.first() {
                    events
                        .player
                        .send(PlayerEvent::CardDropped(card.id, *target));
                } else {
                    events.player.send(PlayerEvent::CardDroppedOnField(card.id));
                }
            } else if let Some(point) = field_pos {
                transform.translation = point;
                transform.rotation = Quat::IDENTITY;
            } else if let Some(point) = point {
                transform.translation = point;
            }
        }
    }
}

/*
fn switch_card_id(
    state: Res<ConsoleUiState>,
    mut query: Query<&mut Visibility, With<CardIdOverlay>>,
) {
    let visible = state.open();
    query.par_iter_mut().for_each(|mut visibility| {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    });
}
*/
