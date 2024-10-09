use super::event::PlayerEvent;
use crate::scene::{
    game::{
        board::{self, AvailableActionList, Board},
        event::InstructionsUpdated,
    },
    translator::{TextPurpose, Translator},
    GlobalState,
};
use bevy::{color::palettes::css, prelude::*};
use bevy_mod_picking::picking_core::Pickable;
use bevy_mod_picking::prelude::*;
use fluent_bundle::{FluentArgs, FluentValue};
use fluent_content::Request;
use kodecks::{
    ability::KeywordAbility,
    action::AvailableAction,
    card::{CardSnapshot, Catalog},
    id::ObjectId,
    text::{parse_text, Section},
};
use kodecks_catalog::CATALOG;
use std::mem::discriminant;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UIEvent>()
            .add_systems(OnEnter(GlobalState::GameLoading), init)
            .add_systems(OnEnter(GlobalState::GameCleanup), cleanup)
            .add_systems(
                Update,
                (
                    update_instructions.run_if(on_event::<InstructionsUpdated>()),
                    update_action_list.run_if(
                        resource_exists_and_changed::<AvailableActionList>
                            .or_else(resource_exists_and_changed::<Board>),
                    ),
                    update_ui.run_if(on_event::<UIEvent>()),
                    (update_card_info, update_card_image, update_buttons)
                        .run_if(resource_exists_and_changed::<UIState>),
                )
                    .run_if(
                        in_state(GlobalState::GameMain)
                            .or_else(in_state(GlobalState::GameResult))
                            .or_else(in_state(GlobalState::GameLoading)),
                    ),
            );
    }
}

#[derive(Event, Clone)]
pub enum UIEvent {
    CardHovered(ObjectId),
}

#[derive(Debug, Resource, Default)]
pub struct UIState {
    pub selected_card: Option<UICardInfo>,
    pub available_buttons: Vec<ActionButton>,
}

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq)]
pub enum ActionButton {
    EndTurn,
    AllAttack,
    Attack(u32),
    Block(u32),
    Continue,
}

impl ActionButton {
    pub fn to_string(self, translator: &Translator) -> String {
        let mut args = FluentArgs::new();
        match self {
            Self::EndTurn => translator.get("end-turn-button").into(),
            Self::AllAttack => translator.get("all-attack-button").into(),
            Self::Attack(n) => {
                args.set("attackers", FluentValue::from(n));
                translator
                    .get(Request::new("attack-button").args(&args))
                    .into()
            }
            Self::Block(n) => {
                args.set("blockers", FluentValue::from(n));
                translator
                    .get(Request::new("block-button").args(&args))
                    .into()
            }
            Self::Continue => translator.get("continue-button").into(),
        }
    }
}

fn update_ui(
    env: Res<board::Environment>,
    mut state: ResMut<UIState>,
    mut events: EventReader<UIEvent>,
) {
    for event in events.read() {
        match event {
            UIEvent::CardHovered(id) => {
                let updated =
                    state.selected_card.as_ref().map(|card| card.snapshot.id) != Some(*id);
                if updated {
                    state.selected_card = env
                        .find_card(*id)
                        .ok()
                        .filter(|card| !card.archetype_id.is_empty())
                        .map(|card| UICardInfo::new(card.clone()));
                }
            }
        }
    }
}

fn update_action_list(
    board: Res<Board>,
    list: Res<AvailableActionList>,
    mut state: ResMut<UIState>,
) {
    state.available_buttons = list
        .iter()
        .filter_map(|action| match action {
            AvailableAction::Attack { .. } => {
                let attackers = board.attackers().count() as u32;
                if attackers == 0 {
                    Some(ActionButton::AllAttack)
                } else {
                    Some(ActionButton::Attack(attackers))
                }
            }
            AvailableAction::Block { .. } => {
                let blockers = board.blocking_pairs().count() as u32;
                if blockers == 0 {
                    Some(ActionButton::Continue)
                } else {
                    Some(ActionButton::Block(blockers))
                }
            }
            AvailableAction::EndTurn => Some(ActionButton::EndTurn),
            _ => None,
        })
        .collect();
}

fn update_card_image(
    state: Res<UIState>,
    mut image_query: Query<(&CardInfo, &mut UiImage)>,
    asset_server: Res<AssetServer>,
) {
    for (_, mut image) in image_query.iter_mut() {
        if let Some(card) = state.selected_card.as_ref() {
            let safe_name = CATALOG[card.snapshot.archetype_id].safe_name;
            *image = UiImage::new(asset_server.load(format!("cards/{}/image.main.png", safe_name)));
        } else {
            *image = UiImage::default();
        }
    }
}

fn update_instructions(
    translator: Res<Translator>,
    mut text_query: Query<(&mut Text, &mut Style), With<InstructionText>>,
    mut events: EventReader<InstructionsUpdated>,
) {
    let (mut text, mut style) = text_query.single_mut();
    for InstructionsUpdated(message) in events.read() {
        if let Some(message) = message {
            let args = message.variables.fluent_args();
            let request = Request::from(&message.id).args(&args);
            *text = Text::from_section(
                translator.get(request),
                TextStyle {
                    color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                    ..translator.style(TextPurpose::CardText)
                },
            );
            style.display = Display::Flex;
        } else {
            style.display = Display::None;
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
            let safe_name = CATALOG[card.snapshot.archetype_id].safe_name;
            let id = format!("card-{safe_name}");
            let name = translator.get(&id);

            match info {
                CardInfo::Name => vec![TextSection::new(
                    name,
                    translator.style(TextPurpose::CardName),
                )],
                CardInfo::Text => card.text_sections(&translator, &CATALOG),
                _ => vec![],
            }
        } else {
            vec![]
        };
    }

    for entity in keyword_query.iter() {
        commands.add(DespawnRecursive { entity });
    }

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
}

fn update_buttons(
    state: Res<UIState>,
    translator: Res<Translator>,
    mut query: Query<(Entity, &mut Style, &ActionButton)>,
    mut text_query: Query<&mut Text>,
    children: Query<&Children>,
) {
    for (entity, mut style, button) in query.iter_mut() {
        let button = state
            .available_buttons
            .iter()
            .find(|&b| discriminant(b) == discriminant(button));
        if let Some(button) = button {
            for child in children.iter_descendants(entity) {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.sections[0].value = button.to_string(&translator);
                }
            }
            style.display = Display::Flex;
        } else {
            style.display = Display::None;
        }
    }
}

#[derive(Component)]
pub struct UIRoot;

#[derive(Component)]
pub struct KeywordList;

#[derive(Component)]
pub struct Keyword;

#[derive(Component)]
struct InstructionText;

#[derive(Clone, Copy, Component)]
pub enum CardInfo {
    Image,
    Name,
    Text,
}

pub fn init(mut commands: Commands, translator: Res<Translator>, asset_server: Res<AssetServer>) {
    commands.insert_resource(UIState::default());

    let slicer = TextureSlicer {
        border: BorderRect::square(5.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    };
    let button = asset_server.load("ui/button.png");
    let button_red = asset_server.load("ui/button-red.png");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.)),
                    ..default()
                },
                ..default()
            },
            Pickable::IGNORE,
            UIRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::FlexEnd,
                        padding: UiRect::all(Val::Px(10.)),
                        column_gap: Val::Px(10.),
                        ..default()
                    },
                    ..default()
                },
                Pickable::IGNORE,
            ));

            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
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
                                    width: Val::Px(260.),
                                    height: Val::Percent(60.0),
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
                                        .with_style(
                                            Style {
                                                margin: UiRect::all(Val::Px(5.)),
                                                ..default()
                                            },
                                        ),
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
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Px(200.),
                                    margin: UiRect::all(Val::Px(10.)),
                                    justify_content: JustifyContent::Center,
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(10.),
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
                                        width: Val::Percent(100.),
                                        padding: UiRect::all(Val::Px(5.)),
                                        ..default()
                                    },
                                    ..default()
                                },))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            "",
                                            translator.style(TextPurpose::CardText),
                                        ),
                                        Label,
                                        InstructionText,
                                    ));
                                });

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
                                    ImageScaleMode::Sliced(slicer.clone()),
                                    On::<Pointer<Click>>::commands_mut(move |_, commands| {
                                        commands.add(move |w: &mut World| {
                                            w.send_event(PlayerEvent::ButtonPressed(
                                                ActionButton::AllAttack,
                                            ));
                                        });
                                    }),
                                    ActionButton::AllAttack,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            "",
                                            translator.style(TextPurpose::Button),
                                        ),
                                        Label,
                                    ));
                                });

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
                                    ImageScaleMode::Sliced(slicer.clone()),
                                    On::<Pointer<Click>>::commands_mut(move |_, commands| {
                                        commands.add(move |w: &mut World| {
                                            w.send_event(PlayerEvent::ButtonPressed(
                                                ActionButton::Attack(0),
                                            ));
                                        });
                                    }),
                                    ActionButton::Attack(0),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            "",
                                            translator.style(TextPurpose::Button),
                                        ),
                                        Label,
                                    ));
                                });

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
                                    ImageScaleMode::Sliced(slicer.clone()),
                                    On::<Pointer<Click>>::commands_mut(move |_, commands| {
                                        commands.add(move |w: &mut World| {
                                            w.send_event(PlayerEvent::ButtonPressed(
                                                ActionButton::Block(0),
                                            ));
                                        });
                                    }),
                                    ActionButton::Block(0),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            "",
                                            translator.style(TextPurpose::Button),
                                        ),
                                        Label,
                                    ));
                                });

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
                                        image: button_red.clone().into(),
                                        ..default()
                                    },
                                    ImageScaleMode::Sliced(slicer.clone()),
                                    On::<Pointer<Click>>::commands_mut(move |_, commands| {
                                        commands.add(move |w: &mut World| {
                                            w.send_event(PlayerEvent::ButtonPressed(
                                                ActionButton::Block(0),
                                            ));
                                        });
                                    }),
                                    ActionButton::Continue,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            "",
                                            translator.style(TextPurpose::Button),
                                        ),
                                        Label,
                                    ));
                                });

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
                                        image: button_red.clone().into(),
                                        ..default()
                                    },
                                    ImageScaleMode::Sliced(slicer.clone()),
                                    On::<Pointer<Click>>::commands_mut(move |_, commands| {
                                        commands.add(move |w: &mut World| {
                                            w.send_event(PlayerEvent::ButtonPressed(
                                                ActionButton::EndTurn,
                                            ));
                                        });
                                    }),
                                    ActionButton::EndTurn,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_section(
                                            "",
                                            translator.style(TextPurpose::Button),
                                        ),
                                        Label,
                                    ));
                                });
                        });
                });

            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::FlexEnd,
                        padding: UiRect::all(Val::Px(10.)),
                        column_gap: Val::Px(10.),
                        ..default()
                    },
                    ..default()
                },
                Pickable::IGNORE,
            ));
        });
}

pub fn cleanup(mut commands: Commands, query: Query<Entity, With<UIRoot>>) {
    commands.remove_resource::<UIState>();
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Debug)]
pub struct UICardInfo {
    pub snapshot: CardSnapshot,
}

impl UICardInfo {
    pub fn new(snapshot: CardSnapshot) -> Self {
        Self { snapshot }
    }

    fn related_abilities(&self, translator: &Translator) -> Vec<KeywordAbility> {
        let mut abilities = self
            .snapshot
            .computed
            .as_ref()
            .map(|attr| attr.abilities.as_ref())
            .unwrap_or_default()
            .to_vec();

        let safe_name = CATALOG[self.snapshot.archetype_id].safe_name;
        abilities.extend(translator.get_related_items(safe_name).abilities);
        abilities.sort();
        abilities.dedup();

        abilities
    }

    fn text_sections(&self, translator: &Translator, catalog: &Catalog) -> Vec<TextSection> {
        let safe_name = catalog[self.snapshot.archetype_id].safe_name;
        let id = format!("card-{safe_name}.text");
        let text = translator.get(&id);

        let abilities = self
            .snapshot
            .computed
            .as_ref()
            .map(|attr| attr.abilities.as_ref())
            .unwrap_or_default();

        let mut sections = abilities
            .iter()
            .flat_map(|ability| {
                let ability_name = format!("ability-{}", ability.to_string().to_lowercase());
                let ability_name = translator.get(&ability_name);
                vec![
                    TextSection::new(
                        ability_name,
                        TextStyle {
                            color: css::GOLD.into(),
                            ..translator.style(TextPurpose::CardText)
                        },
                    ),
                    TextSection::new("  ", translator.style(TextPurpose::CardText)),
                ]
            })
            .collect::<Vec<_>>();

        if !sections.is_empty() {
            sections.push(TextSection::new(
                "\n\n",
                TextStyle {
                    font_size: 10.0,
                    ..default()
                },
            ));
        }

        for section in parse_text(&text) {
            match section {
                Section::Text(text) => {
                    sections.push(TextSection::new(
                        text,
                        translator.style(TextPurpose::CardText),
                    ));
                }
                Section::Card(text) => {
                    sections.push(TextSection::new(
                        text,
                        TextStyle {
                            color: css::LIGHT_BLUE.into(),
                            ..translator.style(TextPurpose::CardText)
                        },
                    ));
                }
                Section::Keyword(ability) => {
                    sections.push(TextSection::new(
                        ability.to_string(),
                        TextStyle {
                            color: css::GOLD.into(),
                            ..translator.style(TextPurpose::CardText)
                        },
                    ));
                }
                Section::Number(n) => {
                    sections.push(TextSection::new(
                        n.to_string(),
                        TextStyle {
                            color: css::LIGHT_PINK.into(),
                            ..translator.style(TextPurpose::CardText)
                        },
                    ));
                }
            }
        }

        sections
    }
}
