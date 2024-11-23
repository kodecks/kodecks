use super::{
    card::{Catalog, UICardInfo, CARD_HEIGHT, CARD_WIDTH},
    translator::{TextPurpose, Translator},
    GlobalState,
};
use crate::{assets::AssetServerExt, save_data::SaveData};
use bevy::{
    color::palettes::css,
    ecs::system::SystemParam,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    text::BreakLineOn,
    ui::RelativeCursorPosition,
};
use bevy_mod_picking::prelude::*;
use kodecks::{
    archetype::ArchetypeId,
    card::{CardEntry, CardSnapshot},
    deck::DeckItem,
};
use kodecks_catalog::CATALOG;
use std::collections::BTreeMap;

pub struct DeckPlugin;

impl Plugin for DeckPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UIState>()
            .add_event::<DeckEvent>()
            .add_event::<UiEvent>()
            .add_systems(OnEnter(GlobalState::DeckMain), init)
            .add_systems(OnExit(GlobalState::DeckMain), cleanup)
            .add_systems(
                Update,
                (
                    mouse_scroll,
                    handle_deck_event.run_if(on_event::<DeckEvent>()),
                    handle_ui_event.run_if(on_event::<UiEvent>()),
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
struct UiRoot;

#[derive(Component)]
struct CollectionItem(ArchetypeId);

#[derive(Component)]
struct UiDeckItem(ArchetypeId);

#[derive(Component)]
struct DeckList;

#[derive(Component)]
struct KeywordList;

#[derive(Component)]
struct Keyword;

#[derive(Debug, Event)]
enum UiEvent {
    CardHovered(ArchetypeId),
    Quit,
}

#[derive(Debug, Event)]
enum DeckEvent {
    RemoveFromDeck(ArchetypeId),
    AddToDeck(ArchetypeId),
}

#[derive(Debug, Resource, Default)]
pub struct UIState {
    pub selected_card: Option<UICardInfo>,
}

fn init(
    mut commands: Commands,
    translator: Res<Translator>,
    save_data: Res<SaveData>,
    asset_server: Res<AssetServer>,
    catalog: Res<Catalog>,
) {
    let slicer = TextureSlicer {
        border: BorderRect::square(2.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 2.0,
    };

    let button_slicer = TextureSlicer {
        border: BorderRect::square(5.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    };

    let button = asset_server.load_with_cache("ui/button-red.png");
    let end = asset_server.load_with_cache("frames/deck_frame_end.png".to_string());

    let mut current_deck = BTreeMap::new();

    let deck = save_data.decks.list.first().unwrap();
    let mut deck = deck
        .cards
        .iter()
        .map(|item| &catalog[item.card.archetype_id])
        .collect::<Vec<_>>();
    deck.sort();
    for card in deck.iter() {
        current_deck
            .entry(card.id)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    let mut inventory = save_data
        .collection
        .cards
        .iter()
        .map(|(id, count)| (&catalog[*id], *count - current_deck.get(id).unwrap_or(&0)))
        .collect::<Vec<_>>();
    inventory.sort_by_key(|(card, _)| *card);

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    column_gap: Val::Px(20.),
                    padding: UiRect::axes(Val::Px(20.), Val::Px(50.)),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                ..default()
            },
            UiRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(260.),
                            height: Val::Percent(100.0),
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
                                        height: Val::Px(60. / CARD_WIDTH * CARD_HEIGHT),
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
                        height: Val::Percent(100.),
                        width: Val::Percent(25.0),
                        min_width: Val::Px(240.0),
                        max_width: Val::Px(480.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            translator.get("deck-label-collection"),
                            translator.style(TextPurpose::CardName),
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(5.)),
                            ..default()
                        }),
                        Label,
                    ));

                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                flex_grow: 1.,
                                width: Val::Percent(100.),
                                height: Val::Px(100.),
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
                                    RelativeCursorPosition::default(),
                                    ScrollingList::default(),
                                ))
                                .with_children(|parent| {
                                    for (archetype, count) in &inventory {
                                        let id = format!("card-{}", archetype.safe_name);
                                        let name = translator.get(&id);
                                        let image = asset_server.load_with_cache(format!(
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
                                                        height: Val::Px(CARD_WIDTH),
                                                        ..default()
                                                    },
                                                    ..default()
                                                },
                                                CollectionItem(archetype_id),
                                                On::<Pointer<Over>>::commands_mut(
                                                    move |_, commands| {
                                                        commands.add(move |w: &mut World| {
                                                            w.send_event(UiEvent::CardHovered(
                                                                archetype_id,
                                                            ));
                                                        });
                                                    },
                                                ),
                                                On::<Pointer<Click>>::commands_mut(
                                                    move |_, commands| {
                                                        commands.add(move |w: &mut World| {
                                                            w.send_event(DeckEvent::AddToDeck(
                                                                archetype_id,
                                                            ));
                                                        });
                                                    },
                                                ),
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
                                                                        style: translator.style(
                                                                            TextPurpose::Number,
                                                                        ),
                                                                    }],
                                                                    justify: JustifyText::Center,
                                                                    linebreak_behavior:
                                                                        BreakLineOn::NoWrap,
                                                                },
                                                                style: Style::default(),
                                                                ..Default::default()
                                                            },
                                                            CollectionItem(archetype_id),
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
                                                                justify_content:
                                                                    JustifyContent::Center,
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
                                                                        style: translator.style(
                                                                            TextPurpose::Button,
                                                                        ),
                                                                    }],
                                                                    justify: JustifyText::Left,
                                                                    linebreak_behavior:
                                                                        BreakLineOn::NoWrap,
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

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        height: Val::Percent(100.),
                        width: Val::Percent(25.0),
                        min_width: Val::Px(240.0),
                        max_width: Val::Px(480.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            translator.get("deck-label-deck"),
                            translator.style(TextPurpose::CardName),
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(5.)),
                            ..default()
                        }),
                        Label,
                    ));

                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                height: Val::Px(100.),
                                width: Val::Percent(100.0),
                                flex_grow: 1.,
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
                                    RelativeCursorPosition::default(),
                                    ScrollingList::default(),
                                    DeckList,
                                ))
                                .with_children(|parent| {
                                    for archetype in &deck {
                                        let id = format!("card-{}", archetype.safe_name);
                                        let name = translator.get(&id);
                                        let image = asset_server.load_with_cache(format!(
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
                                                        height: Val::Px(CARD_WIDTH),
                                                        ..default()
                                                    },
                                                    ..default()
                                                },
                                                UiDeckItem(archetype_id),
                                                On::<Pointer<Over>>::commands_mut(
                                                    move |_, commands| {
                                                        commands.add(move |w: &mut World| {
                                                            w.send_event(UiEvent::CardHovered(
                                                                archetype_id,
                                                            ));
                                                        });
                                                    },
                                                ),
                                                On::<Pointer<Click>>::commands_mut(
                                                    move |_, commands| {
                                                        commands.add(move |w: &mut World| {
                                                            w.send_event(
                                                                DeckEvent::RemoveFromDeck(
                                                                    archetype_id,
                                                                ),
                                                            );
                                                        });
                                                    },
                                                ),
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
                                                                justify_content:
                                                                    JustifyContent::Center,
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
                                                                        style: translator.style(
                                                                            TextPurpose::Button,
                                                                        ),
                                                                    }],
                                                                    justify: JustifyText::Left,
                                                                    linebreak_behavior:
                                                                        BreakLineOn::NoWrap,
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

            parent
                .spawn((NodeBundle {
                    style: Style {
                        height: Val::Percent(100.),
                        width: Val::Px(180.0),
                        justify_content: JustifyContent::End,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(20.),
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|parent| {
                    parent
                        .spawn((
                            ImageBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    height: Val::Px(50.),
                                    padding: UiRect::all(Val::Px(15.)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                image: button.clone().into(),
                                ..default()
                            },
                            ImageScaleMode::Sliced(button_slicer),
                            On::<Pointer<Click>>::commands_mut(move |_, commands| {
                                commands.add(move |w: &mut World| {
                                    w.send_event(UiEvent::Quit);
                                });
                            }),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    translator.get("deck-button-quit"),
                                    translator.style(TextPurpose::Button),
                                ),
                                Label,
                            ));
                        });
                });
        });
}

fn cleanup(mut commands: Commands, ui_query: Query<Entity, With<UiRoot>>) {
    ui_query.iter().for_each(|entity| {
        commands.entity(entity).despawn_recursive();
    });
}

#[derive(Component, Default)]
struct ScrollingList {
    position: f32,
}

fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(
        &mut ScrollingList,
        &RelativeCursorPosition,
        &mut Style,
        &Parent,
        &Node,
    )>,
    query_node: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, pos, mut style, parent, list_node) in &mut query_list {
            if !pos.mouse_over() {
                continue;
            }

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

fn handle_ui_event(
    mut event: EventReader<UiEvent>,
    mut state: ResMut<UIState>,
    mut next_state: ResMut<NextState<GlobalState>>,
    catalog: Res<Catalog>,
) {
    for event in event.read() {
        match event {
            UiEvent::CardHovered(id) => {
                let archetype = &catalog[*id];
                state.selected_card = Some(UICardInfo::new(CardSnapshot::new(archetype)));
            }
            UiEvent::Quit => {
                next_state.set(GlobalState::MenuMain);
            }
        }
    }
}

#[derive(SystemParam)]
pub struct DeckQueries<'w, 's> {
    deck: Query<'w, 's, (Entity, &'static UiDeckItem)>,
    inventory: Query<'w, 's, (&'static mut Text, &'static CollectionItem)>,
    list: Query<'w, 's, Entity, With<DeckList>>,
}

fn handle_deck_event(
    mut commands: Commands,
    mut event: EventReader<DeckEvent>,
    mut queries: DeckQueries,
    mut save_data: ResMut<SaveData>,
    translator: Res<Translator>,
    asset_server: Res<AssetServer>,
    catalog: Res<Catalog>,
) {
    for event in event.read() {
        match event {
            DeckEvent::RemoveFromDeck(id) => {
                if let Some((entity, _)) = queries.deck.iter().find(|(_, item)| item.0 == *id) {
                    commands.entity(entity).despawn_recursive();
                    if let Some((mut text, _)) =
                        queries.inventory.iter_mut().find(|(_, item)| item.0 == *id)
                    {
                        let count = text.sections[0].value.parse::<u32>().unwrap();
                        text.sections[0].value = (count + 1).to_string();
                    }
                    let deck = &mut save_data.decks.list.first_mut().unwrap().cards;
                    if let Some(pos) = deck.iter().position(|item| item.card.archetype_id == *id) {
                        deck.remove(pos);
                    }
                }
            }
            DeckEvent::AddToDeck(id) => {
                if let Some((mut text, _)) =
                    queries.inventory.iter_mut().find(|(_, item)| item.0 == *id)
                {
                    let count = text.sections[0].value.parse::<u32>().unwrap();
                    if count > 0 {
                        text.sections[0].value = (count - 1).to_string();
                    } else {
                        return;
                    }
                }

                let deck = &mut save_data.decks.list.first_mut().unwrap().cards;
                deck.push(DeckItem {
                    card: CardEntry {
                        archetype_id: *id,
                        style: 0,
                    },
                    base_id: None,
                });

                let deck_items = queries
                    .deck
                    .iter()
                    .map(|(_, item)| &catalog[item.0])
                    .collect::<Vec<_>>();

                let archetype = &catalog[*id];
                let id = format!("card-{}", archetype.safe_name);
                let name = translator.get(&id);
                let image = asset_server
                    .load_with_cache(format!("cards/{}/image.main.png#deck", archetype.safe_name));
                let archetype_id = archetype.id;

                let slicer = TextureSlicer {
                    border: BorderRect::square(2.0),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 2.0,
                };

                let end = asset_server.load_with_cache("frames/deck_frame_end.png".to_string());
                let entity = commands
                    .spawn((
                        NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                width: Val::Percent(100.),
                                height: Val::Px(CARD_WIDTH),
                                ..default()
                            },
                            ..default()
                        },
                        UiDeckItem(archetype_id),
                        On::<Pointer<Over>>::commands_mut(move |_, commands| {
                            commands.add(move |w: &mut World| {
                                w.send_event(UiEvent::CardHovered(archetype_id));
                            });
                        }),
                        On::<Pointer<Click>>::commands_mut(move |_, commands| {
                            commands.add(move |w: &mut World| {
                                w.send_event(DeckEvent::RemoveFromDeck(archetype_id));
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
                                                style: translator.style(TextPurpose::Button),
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
                    })
                    .id();

                let index = match deck_items.binary_search(&archetype) {
                    Ok(index) => index,
                    Err(index) => index,
                };

                commands
                    .entity(queries.list.single())
                    .insert_children(index, &[entity]);
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
            let safe_name = CATALOG[card.snapshot.archetype_id].safe_name.clone();
            *image = UiImage::new(
                asset_server.load_with_cache(format!("cards/{}/image.main.png", safe_name)),
            );
        } else {
            *image = UiImage::default();
        }
    }
}

#[derive(SystemParam)]
pub struct CardQueries<'w, 's> {
    text: Query<'w, 's, (&'static CardInfo, &'static mut Text)>,
    keyword: Query<'w, 's, Entity, With<Keyword>>,
    list: Query<'w, 's, Entity, With<KeywordList>>,
}

fn update_card_info(
    mut commands: Commands,
    state: Res<UIState>,
    catalog: Res<Catalog>,
    mut queries: CardQueries,
    asset_server: Res<AssetServer>,
    translator: Res<Translator>,
) {
    for (info, mut text) in queries.text.iter_mut() {
        text.sections = if let Some(card) = state.selected_card.as_ref() {
            let safe_name = CATALOG[card.snapshot.archetype_id].safe_name.clone();
            let id = format!("card-{safe_name}");
            let name = translator.get(&id);

            match info {
                CardInfo::Name => vec![TextSection::new(
                    name,
                    translator.style(TextPurpose::CardName),
                )],
                CardInfo::Text => card.text_sections(&translator, &catalog),
                _ => vec![],
            }
        } else {
            vec![]
        };
    }

    for entity in queries.keyword.iter() {
        commands.add(DespawnRecursive { entity });
    }

    commands
        .entity(queries.list.single())
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
                                image: UiImage::new(asset_server.load_with_cache(format!(
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
}
