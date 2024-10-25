use super::{
    translator::{TextPurpose, Translator},
    GlobalState,
};
use crate::save_data::SaveData;
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    text::BreakLineOn,
};
use bevy_mod_picking::prelude::*;
use kodecks::card::ArchetypeId;
use kodecks_catalog::CATALOG;

pub struct DeckPlugin;

impl Plugin for DeckPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UIState>()
            .add_event::<DeckEvent>()
            .add_systems(OnEnter(GlobalState::DeckMain), init)
            .add_systems(OnExit(GlobalState::DeckMain), cleanup)
            .add_systems(
                Update,
                (
                    mouse_scroll,
                    handle_event,
                    (update_card_image, update_card_info)
                        .run_if(resource_exists_and_changed::<UIState>),
                )
                    .run_if(in_state(GlobalState::DeckMain)),
            );
    }
}

#[derive(Clone, Copy, Component)]
enum CardInfo {
    Image,
    Name,
    Text,
}

#[derive(Component)]
pub struct InventoryItem(ArchetypeId);

#[derive(Component)]
pub struct DeckItem(ArchetypeId);

#[derive(Component)]
pub struct KeywordList;

#[derive(Component)]
pub struct Keyword;

#[derive(Debug, Event)]
enum DeckEvent {
    CardHovered(ArchetypeId),
    RemoveFromDeck(ArchetypeId),
    AddToDeck(ArchetypeId),
}

#[derive(Debug, Resource, Default)]
pub struct UIState {
    pub selected_card: Option<ArchetypeId>,
}

fn init(
    mut commands: Commands,
    translator: Res<Translator>,
    save_data: Res<SaveData>,
    asset_server: Res<AssetServer>,
) {
    let slicer = TextureSlicer {
        border: BorderRect::square(2.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 2.0,
    };

    let end = asset_server.load(format!("frames/deck_frame_end.png"));

    let deck = save_data.decks.get_default("offline").unwrap();
    let mut deck = deck
        .cards
        .iter()
        .map(|item| &CATALOG[item.card.archetype_id])
        .collect::<Vec<_>>();
    deck.sort();

    let mut inventory = save_data
        .inventory
        .cards
        .iter()
        .map(|(id, count)| (&CATALOG[*id], *count))
        .collect::<Vec<_>>();
    inventory.sort_by_key(|(card, _)| *card);

    commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        },))
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(260.),
                            height: Val::Percent(80.0),
                            justify_content: JustifyContent::Start,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Auto,
                                    padding: UiRect::all(Val::Px(10.)),
                                    justify_content: JustifyContent::Start,
                                    align_items: AlignItems::End,
                                    flex_direction: FlexDirection::Row,
                                    ..default()
                                },
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                ImageBundle {
                                    style: Style {
                                        width: Val::Px(60.),
                                        height: Val::Px(60. / 36. * 48.),
                                        padding: UiRect::all(Val::Px(5.)),
                                        ..default()
                                    },
                                    ..default()
                                },
                                CardInfo::Image,
                            ));

                            parent.spawn((
                                TextBundle::from_section(
                                    "",
                                    translator.style(TextPurpose::CardName),
                                )
                                .with_style(Style {
                                    margin: UiRect::all(Val::Px(5.)),
                                    ..default()
                                }),
                                CardInfo::Name,
                                Label,
                            ));
                        });

                    parent
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    padding: UiRect::all(Val::Px(10.)),
                                    justify_content: JustifyContent::Start,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(20.),
                                    ..default()
                                },
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn((NodeBundle {
                                    style: Style {
                                        padding: UiRect::all(Val::Px(5.)),
                                        ..default()
                                    },
                                    ..default()
                                },))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            "",
                                            translator.style(TextPurpose::CardAbility),
                                        ),
                                        Label,
                                        CardInfo::Text,
                                    ));
                                });

                            parent.spawn((
                                NodeBundle {
                                    style: Style {
                                        padding: UiRect::all(Val::Px(5.)),
                                        justify_content: JustifyContent::Start,
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(5.),
                                        ..default()
                                    },
                                    ..default()
                                },
                                KeywordList,
                            ));
                        });
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        height: Val::Percent(80.),
                        width: Val::Px(320.0),
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                ..default()
                            },
                            ScrollingList::default(),
                        ))
                        .with_children(|parent| {
                            for (archetype, count) in &inventory {
                                let id = format!("card-{}", archetype.safe_name);
                                let name = translator.get(&id);
                                let image = asset_server.load(format!(
                                    "cards/{}/image.main.png#deck",
                                    archetype.safe_name
                                ));
                                let archetype_id = archetype.id;

                                parent
                                    .spawn((
                                        NodeBundle {
                                            style: Style {
                                                flex_direction: FlexDirection::Row,
                                                align_items: AlignItems::Center,
                                                width: Val::Percent(100.),
                                                height: Val::Px(36.),
                                                ..default()
                                            },
                                            ..default()
                                        },
                                        InventoryItem(archetype_id),
                                        On::<Pointer<Over>>::commands_mut(move |_, commands| {
                                            commands.add(move |w: &mut World| {
                                                w.send_event(DeckEvent::CardHovered(archetype_id));
                                            });
                                        }),
                                        On::<Pointer<Click>>::commands_mut(move |_, commands| {
                                            commands.add(move |w: &mut World| {
                                                w.send_event(DeckEvent::AddToDeck(archetype_id));
                                            });
                                        }),
                                    ))
                                    .with_children(|parent| {
                                        parent
                                            .spawn(NodeBundle {
                                                style: Style {
                                                    flex_direction: FlexDirection::Row,
                                                    align_items: AlignItems::Center,
                                                    justify_content: JustifyContent::Center,
                                                    width: Val::Px(32.),
                                                    height: Val::Percent(100.),
                                                    ..default()
                                                },
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    TextBundle {
                                                        text: Text {
                                                            sections: vec![TextSection {
                                                                value: count.to_string(),
                                                                style: translator
                                                                    .style(TextPurpose::Number),
                                                            }],
                                                            justify: JustifyText::Center,
                                                            linebreak_behavior: BreakLineOn::NoWrap,
                                                        },
                                                        style: Style::default(),
                                                        ..Default::default()
                                                    },
                                                    InventoryItem(archetype_id),
                                                    Label,
                                                ));
                                            });

                                        parent.spawn((ImageBundle {
                                            style: Style {
                                                width: Val::Px(96.),
                                                height: Val::Percent(100.),
                                                padding: UiRect::all(Val::Px(15.)),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            image: UiImage::new(image.clone()),
                                            ..default()
                                        },));

                                        parent
                                            .spawn((
                                                ImageBundle {
                                                    style: Style {
                                                        width: Val::Px(100.),
                                                        height: Val::Percent(100.),
                                                        padding: UiRect::all(Val::Px(5.)),
                                                        justify_content: JustifyContent::Center,
                                                        align_items: AlignItems::Start,
                                                        flex_grow: 1.,
                                                        overflow: Overflow::clip_x(),
                                                        ..default()
                                                    },
                                                    image: UiImage::new(end.clone()),
                                                    ..default()
                                                },
                                                ImageScaleMode::Sliced(slicer.clone()),
                                            ))
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    TextBundle {
                                                        text: Text {
                                                            sections: vec![TextSection {
                                                                value: name.to_string(),
                                                                style: translator
                                                                    .style(TextPurpose::Button),
                                                            }],
                                                            justify: JustifyText::Left,
                                                            linebreak_behavior: BreakLineOn::NoWrap,
                                                        },
                                                        style: Style {
                                                            width: Val::Percent(100.),
                                                            height: Val::Percent(100.),
                                                            ..default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                    Label,
                                                ));
                                            });
                                    });
                            }
                        });
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        height: Val::Percent(80.),
                        width: Val::Px(320.0),
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                ..default()
                            },
                            ScrollingList::default(),
                        ))
                        .with_children(|parent| {
                            for archetype in &deck {
                                let id = format!("card-{}", archetype.safe_name);
                                let name = translator.get(&id);
                                let image = asset_server.load(format!(
                                    "cards/{}/image.main.png#deck",
                                    archetype.safe_name
                                ));
                                let archetype_id = archetype.id;

                                parent
                                    .spawn((
                                        NodeBundle {
                                            style: Style {
                                                flex_direction: FlexDirection::Row,
                                                width: Val::Percent(100.),
                                                height: Val::Px(36.),
                                                ..default()
                                            },
                                            ..default()
                                        },
                                        DeckItem(archetype_id),
                                        On::<Pointer<Over>>::commands_mut(move |_, commands| {
                                            commands.add(move |w: &mut World| {
                                                w.send_event(DeckEvent::CardHovered(archetype_id));
                                            });
                                        }),
                                        On::<Pointer<Click>>::commands_mut(move |_, commands| {
                                            commands.add(move |w: &mut World| {
                                                w.send_event(DeckEvent::RemoveFromDeck(
                                                    archetype_id,
                                                ));
                                            });
                                        }),
                                    ))
                                    .with_children(|parent| {
                                        parent.spawn((ImageBundle {
                                            style: Style {
                                                width: Val::Px(96.),
                                                height: Val::Percent(100.),
                                                padding: UiRect::all(Val::Px(15.)),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            image: UiImage::new(image.clone()),
                                            ..default()
                                        },));
                                        parent
                                            .spawn((
                                                ImageBundle {
                                                    style: Style {
                                                        width: Val::Px(100.),
                                                        height: Val::Percent(100.),
                                                        padding: UiRect::all(Val::Px(5.)),
                                                        justify_content: JustifyContent::Center,
                                                        align_items: AlignItems::Start,
                                                        flex_grow: 1.,
                                                        overflow: Overflow::clip_x(),
                                                        ..default()
                                                    },
                                                    image: UiImage::new(end.clone()),
                                                    ..default()
                                                },
                                                ImageScaleMode::Sliced(slicer.clone()),
                                            ))
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    TextBundle {
                                                        text: Text {
                                                            sections: vec![TextSection {
                                                                value: name.to_string(),
                                                                style: translator
                                                                    .style(TextPurpose::Button),
                                                            }],
                                                            justify: JustifyText::Left,
                                                            linebreak_behavior: BreakLineOn::NoWrap,
                                                        },
                                                        style: Style {
                                                            width: Val::Percent(100.),
                                                            height: Val::Percent(100.),
                                                            ..default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                    Label,
                                                ));
                                            });
                                    });
                            }
                        });
                });
        });
}

fn cleanup() {}

#[derive(Component, Default)]
struct ScrollingList {
    position: f32,
}

fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    query_node: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut style, parent, list_node) in &mut query_list {
            let items_height = list_node.size().y;
            let container_height = query_node.get(parent.get()).unwrap().size().y;

            let max_scroll = (items_height - container_height).max(0.);

            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };

            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);
            style.top = Val::Px(scrolling_list.position);
        }
    }
}

fn handle_event(
    mut commands: Commands,
    mut event: EventReader<DeckEvent>,
    mut state: ResMut<UIState>,
    deck_query: Query<(Entity, &DeckItem)>,
    mut inventory_query: Query<(&mut Text, &InventoryItem)>,
) {
    for event in event.read() {
        match event {
            DeckEvent::CardHovered(id) => {
                state.selected_card = Some(*id);
            }
            DeckEvent::RemoveFromDeck(id) => {
                if let Some((entity, _)) = deck_query.iter().find(|(_, item)| item.0 == *id) {
                    commands.entity(entity).despawn_recursive();
                    if let Some((mut text, _)) =
                        inventory_query.iter_mut().find(|(_, item)| item.0 == *id)
                    {
                        let count = text.sections[0].value.parse::<u32>().unwrap();
                        if count > 0 {
                            text.sections[0].value = (count + 1).to_string();
                        }
                    }
                }
            }
            DeckEvent::AddToDeck(id) => {
                if let Some((mut text, _)) =
                    inventory_query.iter_mut().find(|(_, item)| item.0 == *id)
                {
                    let count = text.sections[0].value.parse::<u32>().unwrap();
                    if count > 0 {
                        text.sections[0].value = (count - 1).to_string();
                    }
                }
                let deck_items = deck_query
                    .iter()
                    .map(|(_, item)| &CATALOG[item.0])
                    .collect::<Vec<_>>();
                let index = match deck_items.binary_search(&&CATALOG[*id]) {
                    Ok(index) => index,
                    Err(index) => index,
                };
                println!("Card already in deck at position {}", index);
            }
        }
    }
}

fn update_card_image(
    state: Res<UIState>,
    mut image_query: Query<(&CardInfo, &mut UiImage)>,
    asset_server: Res<AssetServer>,
) {
    for (_, mut image) in image_query.iter_mut() {
        if let Some(card) = state.selected_card.as_ref() {
            let safe_name = CATALOG[*card].safe_name;
            *image = UiImage::new(asset_server.load(format!("cards/{}/image.main.png", safe_name)));
        } else {
            *image = UiImage::default();
        }
    }
}

fn update_card_info(
    mut commands: Commands,
    state: Res<UIState>,
    mut text_query: Query<(&CardInfo, &mut Text)>,
    keyword_query: Query<Entity, With<Keyword>>,
    list_query: Query<Entity, With<KeywordList>>,
    asset_server: Res<AssetServer>,
    translator: Res<Translator>,
) {
    for (info, mut text) in text_query.iter_mut() {
        text.sections = if let Some(card) = state.selected_card.as_ref() {
            let safe_name = CATALOG[*card].safe_name;
            let id = format!("card-{safe_name}");
            let name = translator.get(&id);

            match info {
                CardInfo::Name => vec![TextSection::new(
                    name,
                    translator.style(TextPurpose::CardName),
                )],
                //CardInfo::Text => card.text_sections(&translator, &CATALOG),
                _ => vec![],
            }
        } else {
            vec![]
        };
    }

    for entity in keyword_query.iter() {
        commands.add(DespawnRecursive { entity });
    }

    /*
    commands
        .entity(list_query.single())
        .with_children(|parent| {
            if let Some(card) = state.selected_card.as_ref() {
                for ability in card.related_abilities(&translator) {
                    parent
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    justify_content: JustifyContent::Start,
                                    column_gap: Val::Px(5.),
                                    ..default()
                                },
                                ..default()
                            },
                            Keyword,
                        ))
                        .with_children(|parent| {
                            parent.spawn((ImageBundle {
                                style: Style {
                                    width: Val::Px(24.),
                                    height: Val::Px(24.),
                                    padding: UiRect::all(Val::Px(5.)),
                                    ..default()
                                },
                                image: UiImage::new(asset_server.load(format!(
                                    "abilities/{}.png",
                                    ability.to_string().to_lowercase()
                                ))),
                                ..default()
                            },));

                            let ability_name =
                                format!("ability-{}", ability.to_string().to_lowercase());
                            let ability_desc = format!("{ability_name}.description");
                            let ability_name = translator.get(&ability_name);
                            let ability_desc = translator.get(&ability_desc);
                            parent.spawn((
                                TextBundle::from_sections(vec![
                                    TextSection::new(
                                        ability_name,
                                        TextStyle {
                                            color: css::GOLD.into(),
                                            ..translator.style(TextPurpose::CardAbility)
                                        },
                                    ),
                                    TextSection::new(
                                        " - ",
                                        translator.style(TextPurpose::CardAbility),
                                    ),
                                    TextSection::new(
                                        ability_desc,
                                        translator.style(TextPurpose::CardAbility),
                                    ),
                                ]),
                                Label,
                            ));
                        });
                }
            }
        });
        */
}
