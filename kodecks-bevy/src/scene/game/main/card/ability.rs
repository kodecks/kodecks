use super::Card;
use crate::scene::game::board::Environment;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use kodecks::{ability::KeywordAbility, zone::Zone};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbilityOverlay {
    pub index: usize,
    pub ability: Option<KeywordAbility>,
}

#[derive(AssetCollection, Resource)]
pub struct AbilityAssets {
    #[asset(standard_material)]
    #[asset(path = "abilities/toxic.png")]
    toxic: Handle<StandardMaterial>,

    #[asset(standard_material)]
    #[asset(path = "abilities/volatile.png")]
    volatile: Handle<StandardMaterial>,
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
                            *material = match ability {
                                KeywordAbility::Toxic => assets.toxic.clone(),
                                KeywordAbility::Volatile => assets.volatile.clone(),
                                _ => continue,
                            };
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
