use std::cmp::Ordering;

use super::CARD_WIDTH;
use super::{Card, CardBundleBuilder};
use crate::scene::game::board::Environment;
use bevy::prelude::*;
use image::Rgba;
use kodecks::zone::ZoneKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumberOverlayKey {
    Cost {
        cost: u8,
    },
    Field {
        color: Rgba<u8>,
        power: Option<u32>,
        shields: Option<u8>,
        diff: Ordering,
    },
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberOverlay {
    Cost,
    Field,
}

pub fn update_number_overlay(
    env: Res<Environment>,
    card_query: Query<(&Card, Entity)>,
    children: Query<&mut Children>,
    mut overlay_query: Query<(
        &mut Visibility,
        &mut Handle<StandardMaterial>,
        &mut Transform,
        &NumberOverlay,
    )>,
    mut builder: CardBundleBuilder,
) {
    for (card, entity) in card_query.iter() {
        let children = children.get(entity).unwrap();
        for child in children.iter() {
            if let Ok((mut visibility, mut material, mut transform, overlay)) =
                overlay_query.get_mut(*child)
            {
                if *overlay == NumberOverlay::Cost {
                    if let Ok(zone) = env.find_zone(card.id) {
                        if let Some(item) = env
                            .players
                            .get(zone.player)
                            .unwrap()
                            .hand
                            .iter()
                            .find(|item| item.card.id == card.id)
                        {
                            *visibility = Visibility::Visible;
                            *material = builder.number(overlay, &item.card);
                        } else {
                            *visibility = Visibility::Hidden;
                        }
                        if zone.kind == ZoneKind::Hand {
                            *visibility = Visibility::Visible;
                        } else {
                            *visibility = Visibility::Hidden;
                        }
                        *material = builder.number(overlay, card);
                    }
                } else if let Ok(zone) = env.find_zone(card.id) {
                    if zone.kind == ZoneKind::Field {
                        *visibility = Visibility::Visible;
                    } else {
                        *visibility = Visibility::Hidden;
                    }

                    transform.translation.z = if zone.player == env.player {
                        1.0 / CARD_WIDTH * 14.0
                    } else {
                        -1.0 / CARD_WIDTH * 14.0
                    };

                    *material = builder.number(overlay, card);
                }
            }
        }
    }
}
