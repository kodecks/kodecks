use super::{Environment, GameCondition};
use crate::{
    action::{Action, AvailableActionList, PlayerAvailableActions},
    card::CardSnapshot,
    env::Report,
    error::Error,
    field::{FieldItem, FieldState},
    id::ObjectId,
    phase::Phase,
    player::{LocalPlayerState, PlayerList, PlayerZone},
    stack::{LocalStackItem, Stack},
    zone::CardZone,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct LocalEnvironment {
    pub player: u8,
    pub turn: u32,
    pub timestamp: u64,
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

    pub fn next_id(&self, player: u8) -> u8 {
        self.players.next_id(player)
    }

    pub fn next_player(&self, player: u8) -> &LocalPlayerState {
        self.players.next_player(player)
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
            self.timestamp += 1;
        }
        Report {
            available_actions: Some(PlayerAvailableActions {
                player: self.players.player_in_turn().id,
                actions: AvailableActionList::new(),
                instructions: None,
                message_dialog: None,
            }),
            logs: vec![],
            condition: self.game_condition,
            timestamp: self.timestamp,
        }
    }
}

impl Environment {
    pub fn local(&self, receiver: u8) -> LocalEnvironment {
        let players = PlayerList::new(
            self.state.players.player_in_turn().id,
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
            timestamp: self.timestamp,
        }
    }
}
