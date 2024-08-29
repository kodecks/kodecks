use super::{Environment, GameCondition};
use crate::{
    action::{Action, AvailableActionList, PlayerAvailableActions},
    card::CardSnapshot,
    error::Error,
    field::{FieldItem, FieldState},
    game::Report,
    id::ObjectId,
    phase::Phase,
    player::{LocalPlayerState, PlayerId, PlayerList, PlayerZone},
    stack::{LocalStackItem, Stack},
    zone::CardZone,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct LocalEnvironment {
    pub player: PlayerId,
    pub turn: u32,
    pub players: PlayerList<LocalPlayerState>,
    pub phase: Phase,
    pub stack: Stack<LocalStackItem>,
    pub game_condition: GameCondition,
}

impl LocalEnvironment {
    pub fn find_card(&self, card: ObjectId) -> Result<&CardSnapshot, Error> {
        self.players
            .iter()
            .filter_map(|player| player.find_card(card))
            .next()
            .ok_or(Error::CardNotFound { id: card })
    }

    pub fn find_zone(&self, card: ObjectId) -> Result<PlayerZone, Error> {
        for player in self.players.iter() {
            if let Some(zone) = player.find_zone(card) {
                return Ok(PlayerZone {
                    player: player.id,
                    zone,
                });
            }
        }
        Err(Error::CardNotFound { id: card })
    }

    pub fn cards(&self) -> impl Iterator<Item = &CardSnapshot> {
        self.players.iter().flat_map(|player| player.cards())
    }

    pub fn next_player(&self, player: PlayerId) -> PlayerId {
        self.players.next(player)
    }

    pub fn tick(&mut self, action: Action) -> Report {
        if let Action::CastCard { card, .. } = action {
            let player = self.players.get_mut(self.player);
            if let Some(card) = CardZone::remove(&mut player.hand, card) {
                player.field.push(FieldItem {
                    card,
                    state: FieldState::Active,
                    battle: None,
                });
            }
        }
        Report {
            available_actions: Some(PlayerAvailableActions {
                player: self.players.player_in_turn(),
                actions: AvailableActionList::new(),
                message_dialog: None,
            }),
            logs: vec![],
            condition: self.game_condition,
        }
    }
}

impl Environment {
    pub fn local(&self, receiver: PlayerId) -> LocalEnvironment {
        let players = PlayerList::new(
            self.state.players.player_in_turn(),
            self.state
                .players
                .iter()
                .map(|player| LocalPlayerState::new(player, receiver == player.id)),
        );
        let stack = self.stack.iter().map(|item| item.clone().into()).collect();
        LocalEnvironment {
            player: receiver,
            turn: self.state.turn,
            players,
            phase: self.state.phase.clone(),
            stack,
            game_condition: self.game_condition,
        }
    }
}
