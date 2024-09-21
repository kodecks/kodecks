use super::Environment;
use crate::{
    action::{AvailableAction, PlayerAvailableActions},
    filter_vec,
    message::Message,
    phase::Phase,
    variable::VariableList,
    zone::CardZone,
};

impl Environment {
    pub fn available_actions(&self) -> Option<PlayerAvailableActions> {
        let used_hexes = self.state.players.iter().any(|player| {
            player
                .field
                .items()
                .any(|item| item.card.computed().is_hex())
        });
        if used_hexes {
            return None;
        }

        let active_player = self.state.players.player_in_turn();
        let attackers = active_player
            .field
            .active_cards()
            .map(|c| c.id())
            .collect::<Vec<_>>();
        if let Phase::Main = &self.state.phase {
            if !self.stack.is_empty() {
                return None;
            }

            let castable_cards = active_player
                .castable_cards(&self.state)
                .collect::<Vec<_>>();
            Some(PlayerAvailableActions {
                player: active_player.id,
                actions: filter_vec![
                    if castable_cards.is_empty() {
                        None
                    } else {
                        Some(AvailableAction::CastCard {
                            cards: castable_cards,
                        })
                    },
                    if !attackers.is_empty() {
                        Some(AvailableAction::Attack { attackers })
                    } else {
                        None
                    },
                    Some(AvailableAction::EndTurn),
                ]
                .into_iter()
                .collect(),
                instructions: None,
                message_dialog: None,
            })
        } else if let Phase::Block = &self.state.phase {
            active_player.field.attacking_cards().next()?;

            let player_in_action = self.state.players.next_player(active_player.id);
            let blockers = player_in_action
                .field
                .active_cards()
                .map(|card| card.id())
                .collect::<Vec<_>>();
            let castable_cards = player_in_action
                .castable_cards(&self.state)
                .collect::<Vec<_>>();

            Some(PlayerAvailableActions {
                player: player_in_action.id,
                actions: filter_vec![
                    Some(AvailableAction::Block { blockers }),
                    if castable_cards.is_empty() {
                        None
                    } else {
                        Some(AvailableAction::CastCard {
                            cards: castable_cards,
                        })
                    },
                ]
                .into_iter()
                .collect(),
                instructions: None,
                message_dialog: None,
            })
        } else if matches!(self.state.phase, Phase::End)
            && active_player.hand.len() > self.state.regulation.max_hand_size as usize
        {
            Some(PlayerAvailableActions {
                player: active_player.id,
                actions: vec![AvailableAction::SelectCard {
                    cards: active_player.hand.iter().map(|card| card.id()).collect(),
                    score_factor: -1,
                }]
                .into_iter()
                .collect(),
                instructions: Some(Message {
                    id: "message-discard-excess-cards".to_string(),
                    variables: VariableList::new()
                        .set("maxHandSize", self.state.regulation.max_hand_size as i32),
                }),
                message_dialog: None,
            })
        } else {
            None
        }
    }
}
