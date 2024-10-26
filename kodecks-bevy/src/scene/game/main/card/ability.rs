use super::Card;
use crate::{assets::AssetServerExt, scene::game::board::Environment};
use bevy::{prelude::*, utils::HashMap};
use kodecks::{ability::KeywordAbility, zone::Zone};
use strum::IntoEnumIterator;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbilityOverlay {
    pub index: usize,
    pub ability: Option<KeywordAbility>,
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let assets = KeywordAbility::iter()
        .map(|ability| {
            let image = asset_server.load_with_cache(format!(
                "abilities/{}.png",
                ability.to_string().to_lowercase()
            ));
            (
                ability,
                materials.add(StandardMaterial {
                    base_color_texture: Some(image),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                }),
            )
        })
        .collect();
    commands.insert_resource(AbilityAssets { materials: assets });
}

#[derive(Resource)]
pub struct AbilityAssets {
    materials: HashMap<KeywordAbility, Handle<StandardMaterial>>,
}

pub fn update_ability_overlay(
    env: Res<Environment>,
    card_query: Query<(&Card, Entity)>,
    children: Query<&mut Children>,
    mut overlay_query: Query<(
        &mut Visibility,
        &mut Handle<StandardMaterial>,
        &mut AbilityOverlay,
    )>,
    assets: Res<AbilityAssets>,
) {
    for (card, entity) in card_query.iter() {
        let children = children.get(entity).unwrap();
        if let Some(computed) = &card.computed {
            for child in children.iter() {
                if let Ok((mut visibility, mut material, mut overlay)) =
                    overlay_query.get_mut(*child)
                {
                    let ability = computed.abilities.as_ref().get(overlay.index);

                    if let Ok(zone) = env.find_zone(card.id) {
                        overlay.ability = ability.cloned().filter(|_| zone.zone == Zone::Field);

                        if let Some(ability) = overlay.ability {
                            *material = assets.materials[&ability].clone();
                            *visibility = Visibility::Visible;
                        } else {
                            *visibility = Visibility::Hidden;
                        }
                    }
                }
            }
        }
    }
}
