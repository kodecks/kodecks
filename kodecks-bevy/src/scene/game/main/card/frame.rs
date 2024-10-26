use super::Card;
use crate::{
    assets::AssetServerExt,
    scene::game::board::{AvailableActionList, Board},
};
use bevy::prelude::*;
use kodecks::field::FieldState;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardFrame {
    Shadow,
    SmallOverlay,
}

#[derive(Resource)]
pub struct CardFrameAssets {
    card_shadow: Handle<StandardMaterial>,
    card_active: Handle<StandardMaterial>,
    card_normal: Handle<StandardMaterial>,
    card_select_small: Handle<StandardMaterial>,
    card_attack: Handle<StandardMaterial>,
    card_select: Handle<StandardMaterial>,
    card_attack_active: Handle<StandardMaterial>,
    card_block: Handle<StandardMaterial>,
    card_block_active: Handle<StandardMaterial>,
    card_exhausted: Handle<StandardMaterial>,
}

pub fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let card_shadow = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.5),
        base_color_texture: Some(asset_server.load_with_cache("frames/frame.png")),
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        unlit: true,
        ..default()
    });

    let card_active = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.8, 1.0),
        base_color_texture: Some(asset_server.load_with_cache("frames/frame.png")),
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        emissive: LinearRgba::rgb(0.0, 0.0, 100.0),
        ..default()
    });

    let card_normal = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_cache("frames/compact.png")),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let card_select_small = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_cache("frames/compact.png")),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::rgb(0.0, 50.0, 0.0),
        ..default()
    });

    let card_attack = materials.add(StandardMaterial {
        base_color_texture: Some(
            asset_server.load_with_cache("frames/compact_attack_inactive.png"),
        ),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let select_texture_handle = asset_server.load_with_cache("frames/frame.png");
    let card_select = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 1.0),
        base_color_texture: Some(select_texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::rgb(0.0, 50.0, 0.0),
        ..default()
    });

    let card_attack_active = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.0, 0.0, 1.0),
        base_color_texture: Some(asset_server.load_with_cache("frames/compact_attack.png")),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::rgb(50.0, 0.0, 0.0),
        ..default()
    });

    let card_block = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_cache("frames/compact_block_inactive.png")),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let card_block_active = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 1.0, 1.0),
        base_color_texture: Some(asset_server.load_with_cache("frames/compact_block.png")),
        alpha_mode: AlphaMode::Blend,
        emissive: LinearRgba::rgb(0.0, 0.0, 50.0),
        ..default()
    });

    let exhausted_texture_handle = asset_server.load_with_cache("frames/compact_exhausted.png");
    let card_exhausted = materials.add(StandardMaterial {
        base_color_texture: Some(exhausted_texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let assets = CardFrameAssets {
        card_shadow,
        card_active,
        card_normal,
        card_select_small,
        card_attack,
        card_select,
        card_attack_active,
        card_block,
        card_block_active,
        card_exhausted,
    };

    commands.insert_resource(assets);
}

pub fn update_frame_overlay(
    board: Res<Board>,
    list: Res<AvailableActionList>,
    mut query: Query<(&mut Handle<StandardMaterial>, &Parent, &CardFrame)>,
    card_query: Query<&Card>,
    assets: Res<CardFrameAssets>,
) {
    for (mut material, parent, frame) in query.iter_mut() {
        let id = card_query.get(parent.get()).unwrap().0.id;
        let exhausted = board
            .player_field
            .iter()
            .chain(board.opponent_field.iter())
            .any(|(card, state)| card == &id && state == &FieldState::Exhausted);
        let selectable = list.selectable_cards().iter().any(|card| *card == id);
        if *frame == CardFrame::Shadow {
            let castable = list.castable_cards().iter().any(|card| *card == id);
            *material = if castable {
                assets.card_active.clone()
            } else if selectable {
                assets.card_select.clone()
            } else {
                assets.card_shadow.clone()
            };
        } else {
            *material = if selectable {
                assets.card_select_small.clone()
            } else if board.attackers().any(|attacker| *attacker == id) {
                assets.card_attack_active.clone()
            } else if list.attackers().contains(&id) {
                assets.card_attack.clone()
            } else if board.blocking_pairs().any(|(_, blocker)| *blocker == id) {
                assets.card_block_active.clone()
            } else if list.blockers().contains(&id) {
                assets.card_block.clone()
            } else if exhausted {
                assets.card_exhausted.clone()
            } else {
                assets.card_normal.clone()
            };
        }
    }
}
