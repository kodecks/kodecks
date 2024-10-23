use super::{
    game::mode::{GameMode, GameModeKind},
    translator::{TextPurpose, Translator},
    GlobalState,
};
use crate::{app_version, config::GlobalConfig, save_data};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use kodecks::{deck::DeckList, regulation::Regulation};
use kodecks_catalog::decks::{blue_deck, red_deck};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MenuEvent>()
            .add_systems(OnEnter(GlobalState::MenuMain), init)
            .add_systems(OnExit(GlobalState::MenuMain), cleanup)
            .add_systems(Update, handle_menu_events.run_if(on_event::<MenuEvent>()));
    }
}

#[derive(Component)]
struct UiRoot;

#[derive(Event)]
enum MenuEvent {
    StartBotMatch { deck_list: DeckList },
    StartRandomMatch,
    EditDeck,
}

fn init(mut commands: Commands, translator: Res<Translator>, asset_server: Res<AssetServer>) {
    let slicer = TextureSlicer {
        border: BorderRect::square(5.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    };
    let button = asset_server.load("ui/button.png");

    commands
        .spawn((
            NodeBundle {
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
            },
            UiRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(50.0),
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            text: Text {
                                sections: vec![TextSection {
                                    value: "Kodecks".to_string(),
                                    style: translator.style(TextPurpose::Result),
                                }],
                                ..Default::default()
                            },
                            ..default()
                        },
                        Label,
                    ));
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(50.0),
                        justify_content: JustifyContent::Start,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(20.),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            ImageBundle {
                                style: Style {
                                    width: Val::Px(280.),
                                    height: Val::Px(50.),
                                    padding: UiRect::all(Val::Px(15.)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                image: button.clone().into(),
                                ..default()
                            },
                            ImageScaleMode::Sliced(slicer.clone()),
                            On::<Pointer<Click>>::commands_mut(move |_, commands| {
                                commands.add(move |w: &mut World| {
                                    w.send_event(MenuEvent::StartBotMatch {
                                        deck_list: red_deck(),
                                    });
                                });
                            }),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    translator.get("menu-button-cpu-match-1"),
                                    translator.style(TextPurpose::Button),
                                ),
                                Label,
                            ));
                        });

                    parent
                        .spawn((
                            ImageBundle {
                                style: Style {
                                    width: Val::Px(280.),
                                    height: Val::Px(50.),
                                    padding: UiRect::all(Val::Px(15.)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                image: button.clone().into(),
                                ..default()
                            },
                            ImageScaleMode::Sliced(slicer.clone()),
                            On::<Pointer<Click>>::commands_mut(move |_, commands| {
                                commands.add(move |w: &mut World| {
                                    w.send_event(MenuEvent::StartBotMatch {
                                        deck_list: blue_deck(),
                                    });
                                });
                            }),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    translator.get("menu-button-cpu-match-2"),
                                    translator.style(TextPurpose::Button),
                                ),
                                Label,
                            ));
                        });

                    parent
                        .spawn((
                            ImageBundle {
                                style: Style {
                                    width: Val::Px(280.),
                                    height: Val::Px(50.),
                                    padding: UiRect::all(Val::Px(15.)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                image: button.clone().into(),
                                ..default()
                            },
                            ImageScaleMode::Sliced(slicer.clone()),
                            On::<Pointer<Click>>::commands_mut(move |_, commands| {
                                commands.add(move |w: &mut World| {
                                    w.send_event(MenuEvent::StartRandomMatch);
                                });
                            }),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    translator.get("menu-button-random-match"),
                                    translator.style(TextPurpose::Button),
                                ),
                                Label,
                            ));
                        });

                    parent
                        .spawn((
                            ImageBundle {
                                style: Style {
                                    width: Val::Px(280.),
                                    height: Val::Px(50.),
                                    padding: UiRect::all(Val::Px(15.)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                image: button.clone().into(),
                                ..default()
                            },
                            ImageScaleMode::Sliced(slicer.clone()),
                            On::<Pointer<Click>>::commands_mut(move |_, commands| {
                                commands.add(move |w: &mut World| {
                                    w.send_event(MenuEvent::EditDeck);
                                });
                            }),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    translator.get("menu-button-deck-edit"),
                                    translator.style(TextPurpose::Button),
                                ),
                                Label,
                            ));
                        });
                });
        });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::Start,
                    ..default()
                },
                ..default()
            },
            UiRoot,
            Pickable::IGNORE,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    app_version(),
                    TextStyle {
                        font_size: 20.0,
                        color: Color::linear_rgb(0.5, 0.5, 0.5),
                        ..translator.style(TextPurpose::Title)
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(10.)),
                    ..default()
                }),
                Pickable::IGNORE,
            ));
        });
}

fn handle_menu_events(
    mut commands: Commands,
    mut events: EventReader<MenuEvent>,
    mut next_state: ResMut<NextState<GlobalState>>,
    save_data: Res<save_data::SaveData>,
    config: Res<GlobalConfig>,
) {
    let event = if let Some(event) = events.read().next() {
        event
    } else {
        return;
    };

    let (kind, deck) = match &event {
        MenuEvent::StartBotMatch { deck_list } => (
            GameModeKind::BotMatch {
                bot_deck: deck_list.clone(),
            },
            save_data.decks.get_default("offline").unwrap(),
        ),
        MenuEvent::StartRandomMatch => (
            GameModeKind::RandomMatch {
                server: config.server.clone(),
            },
            save_data.decks.get_default("online").unwrap(),
        ),
        MenuEvent::EditDeck => {
            next_state.set(GlobalState::DeckMain);
            return;
        }
    };

    let mode = GameMode {
        regulation: Regulation::STANDARD,
        player_deck: deck.clone(),
        kind,
    };

    commands.insert_resource(mode);
    next_state.set(GlobalState::GameInit);
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<UiRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
