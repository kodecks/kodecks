use super::ui::ActionButton;
use crate::scene::game::board::{AvailableActionList, Board, Environment};
use crate::scene::game::server::SendCommand;
use crate::scene::GlobalState;
use bevy::prelude::*;
use kodecks::action::{Action, AvailableAction};
use kodecks::id::ObjectId;

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerEventFinished>()
            .add_event::<PlayerEvent>()
            .add_systems(
                Update,
                (
                    handle_player_events.run_if(on_event::<PlayerEvent>()),
                    check_game_condition.run_if(resource_exists_and_changed::<Environment>),
                )
                    .run_if(in_state(GlobalState::GameMain)),
            );
    }
}

#[derive(Event)]
pub struct PlayerEventFinished;

#[derive(Debug, Event, Clone)]
pub enum PlayerEvent {
    ButtonPressed(ActionButton),
    CardDroppedOnField(ObjectId),
    CardDropped(ObjectId, ObjectId),
    CardClicked(ObjectId),
}

fn handle_player_events(
    mut commands: Commands,
    mut board: ResMut<Board>,
    env: Res<Environment>,
    list: Res<AvailableActionList>,
    mut events: EventReader<PlayerEvent>,
    mut finished: EventWriter<PlayerEventFinished>,
) {
    let action = events.read().find_map(|event| match event {
        PlayerEvent::ButtonPressed(button) => match button {
            ActionButton::EndTurn => Some(Action::EndTurn),
            ActionButton::Block(_) | ActionButton::Continue => Some(Action::Block {
                pairs: board.blocking_pairs().copied().collect(),
            }),
            ActionButton::Attack(_) => Some(Action::Attack {
                attackers: board.attackers().copied().collect(),
            }),
            ActionButton::AllAttack => list.iter().find_map(|action| match action {
                AvailableAction::Attack { attackers } => Some(Action::Attack {
                    attackers: attackers.clone(),
                }),
                _ => None,
            }),
        },
        PlayerEvent::CardDroppedOnField(dropped) => {
            if list.castable_cards().iter().any(|card| *card == *dropped) {
                Some(Action::CastCard { card: *dropped })
            } else {
                None
            }
        }
        PlayerEvent::CardDropped(dropped, target) => {
            if list.castable_cards().iter().any(|card| *card == *dropped) {
                Some(Action::CastCard { card: *dropped })
            } else {
                board.toggle_blocker(*dropped, Some(*target));
                None
            }
        }
        PlayerEvent::CardClicked(card) => {
            if list.attackers().contains(card) {
                board.toggle_attacker(*card);
            } else if list.blockers().contains(card) {
                board.toggle_blocker(*card, None);
            } else if list.selectable_cards().contains(card) {
                return Some(Action::SelectCard { card: *card });
            }
            None
        }
    });

    if let Some(action) = &action {
        commands.add(SendCommand(action.clone()));

        let mut env = env.clone();
        let available_actions = env.tick(action.clone()).available_actions;

        if matches!(
            action,
            Action::Attack { .. } | Action::Block { .. } | Action::EndTurn | Action::Concede
        ) {
            commands.insert_resource(AvailableActionList::new(
                available_actions
                    .as_ref()
                    .map(|actions| actions.actions.clone())
                    .unwrap_or_default(),
                0,
            ));
        }

        board.update(&env);
        commands.insert_resource::<Environment>(env.into());
    } else {
        finished.send(PlayerEventFinished);
    }
}

fn check_game_condition(mut next_state: ResMut<NextState<GlobalState>>, env: Res<Environment>) {
    if env.game_condition.is_ended() {
        next_state.set(GlobalState::GameResult);
    }
}
