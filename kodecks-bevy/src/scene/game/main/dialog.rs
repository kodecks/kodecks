use crate::{
    assets::AssetServerExt,
    scene::{
        game::{board::AvailableActionList, event::MessageDialogUpdated, server::SendCommand},
        translator::{TextPurpose, Translator},
        GlobalState,
    },
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use fluent_content::Request;
use kodecks::{
    action::Action,
    message::{MessageBox, MessageBoxPosition},
};

pub struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_event.run_if(on_event::<MessageDialogUpdated>()),
                update_ui.run_if(resource_changed_or_removed::<DialogMessages>()),
                update_text.run_if(resource_exists::<DialogMessages>),
            )
                .run_if(in_state(GlobalState::GameMain)),
        )
        .add_systems(OnEnter(GlobalState::GameLoading), init)
        .add_systems(OnEnter(GlobalState::GameCleanup), cleanup);
    }
}

#[derive(Resource, Default)]
pub struct DialogMessages {
    pub messages: Vec<MessageBox>,
}

#[derive(Component)]
struct DialogBackground;

#[derive(Component)]
struct DialogText;

fn handle_event(mut commands: Commands, mut event: EventReader<MessageDialogUpdated>) {
    if let Some(MessageDialogUpdated(instruction)) = event.read().next() {
        if let Some(instruction) = instruction {
            commands.insert_resource(DialogMessages {
                messages: instruction.messages.iter().rev().cloned().collect(),
            });
        } else {
            commands.remove_resource::<DialogMessages>();
        }
    }
}

fn update_ui(
    available_actions: Res<AvailableActionList>,
    messages: Option<Res<DialogMessages>>,
    mut bg_query: Query<(&mut Visibility, &mut Style, &mut Pickable), With<DialogBackground>>,
    mut text_query: Query<&mut Text, With<DialogText>>,
    translator: Res<Translator>,
) {
    let (mut visibility, mut style, mut pick) = bg_query.single_mut();
    if let Some(messages) = messages {
        *visibility = Visibility::Visible;
        if messages.messages.len() > 1 || available_actions.can_continue() {
            *pick = Default::default();
        } else {
            *pick = Pickable::IGNORE;
        }
        if let Some(message_box) = messages.messages.last() {
            style.justify_content = if message_box.position == MessageBoxPosition::Top {
                JustifyContent::Start
            } else {
                JustifyContent::End
            };
            let mut text = text_query.single_mut();
            let args = message_box.message.variables.fluent_args();
            let request = Request::from(&message_box.message.id).args(&args);
            *text = Text::from_sections(translator.get(request).chars().map(|c| TextSection {
                value: c.to_string(),
                style: TextStyle {
                    color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                    ..translator.style(TextPurpose::Dialog)
                },
            }));
        }
    } else {
        *visibility = Visibility::Hidden;
    }
}

// Based on https://gist.github.com/rparrett/3aa4761a43e70bd99c69c26dd3000df1
fn update_text(
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
    mut text_query: Query<&mut Text, With<DialogText>>,
) {
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(0.01, TimerMode::Repeating));
    if !timer.tick(time.delta()).just_finished() {
        return;
    }

    let mut text = text_query.single_mut();
    if let Some(section) = text
        .sections
        .iter_mut()
        .find(|s| s.style.color.alpha() == 0.0)
    {
        section.style.color.set_alpha(1.0);
    }
}

fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    let slicer = TextureSlicer {
        border: BorderRect::square(5.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    };
    let image = asset_server.load_with_cache("ui/button.png");

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(20.)),
                    ..default()
                },
                z_index: ZIndex::Global(1),
                visibility: Visibility::Hidden,
                ..default()
            },
            Pickable::default(),
            On::<Pointer<Click>>::commands_mut(move |_, commands| {
                commands.add(move |w: &mut World| {
                    let mut messages = w.resource_mut::<DialogMessages>();
                    if messages.messages.len() > 1 {
                        messages.messages.pop();
                    } else {
                        w.commands().add(SendCommand(Action::Continue));
                    }
                });
            }),
            DialogBackground,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ImageBundle {
                        style: Style {
                            width: Val::Px(400.0),
                            height: Val::Px(140.0),
                            padding: UiRect::all(Val::Px(10.)),
                            margin: UiRect::all(Val::Px(40.)),
                            ..default()
                        },
                        image: image.clone().into(),
                        ..default()
                    },
                    ImageScaleMode::Sliced(slicer.clone()),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(15.)),
                                ..default()
                            },
                            ..Default::default()
                        },
                        Label,
                        DialogText,
                    ));
                });
        });
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<DialogBackground>>) {
    commands.remove_resource::<DialogMessages>();
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
