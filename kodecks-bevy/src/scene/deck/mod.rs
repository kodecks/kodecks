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
use kodecks_catalog::CATALOG;

pub struct DeckPlugin;

impl Plugin for DeckPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalState::DeckMain), init)
            .add_systems(OnExit(GlobalState::DeckMain), cleanup)
            .add_systems(Update, mouse_scroll.run_if(in_state(GlobalState::DeckMain)));
    }
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
    deck.sort_by_key(|card| card.attribute.cost);

    commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        },))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        height: Val::Percent(50.),
                        width: Val::Px(320.0),
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                    background_color: Color::srgb(1.0, 1.0, 1.0).into(),
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

                                parent
                                    .spawn(NodeBundle {
                                        style: Style {
                                            flex_direction: FlexDirection::Row,
                                            width: Val::Percent(100.),
                                            height: Val::Px(36.),
                                            ..default()
                                        },
                                        ..default()
                                    })
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
