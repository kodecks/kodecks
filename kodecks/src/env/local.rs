use super::{EndgameState, Environment};
use crate::{
    action::{Action, AvailableActionList, PlayerAvailableActions},
    card::CardSnapshot,
    env::Report,
    error::ActionError,
    id::ObjectId,
    phase::Phase,
    player::{LocalPlayerState, PlayerList, Zone},
    stack::{LocalStackItem, Stack},
    zone::CardZone,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct LocalEnvironment {
    pub player: u8,
    pub turn: u16,
    pub timestamp: u32,
    pub players: PlayerList<LocalPlayerState>,
    pub phase: Phase,
    pub stack: Stack<LocalStackItem>,
    pub endgame: EndgameState,
}

impl LocalEnvironment {
    pub fn find_card(&self, card: ObjectId) -> Result<&CardSnapshot, ActionError> {
        self.players
            .iter()
            .filter_map(|player| player.find_card(card))
            .next()
            .ok_or(ActionError::CardNotFound { id: card })
    }

    pub fn find_zone(&self, card: ObjectId) -> Result<Zone, ActionError> {
        for player in self.players.iter() {
            if let Some(zone) = player.find_zone(card) {
                return Ok(Zone {
                    player: player.id,
                    kind: zone,
                });
            }
        }
        Err(ActionError::CardNotFound { id: card })
    }

    pub fn cards(&self) -> impl Iterator<Item = &CardSnapshot> {
        self.players.iter().flat_map(|player| player.cards())
    }

    pub fn next_id(&self, player: u8) -> Result<u8, ActionError> {
        self.players.next_id(player)
    }

    pub fn next_player(&self, player: u8) -> Result<&LocalPlayerState, ActionError> {
        self.players.next_player(player)
    }

    pub fn tick(&mut self, action: Action) -> Report {
        if let Action::CastCard { card, .. } = action {
            if let Ok(player) = self.players.get_mut(self.player) {
                if let Some(card) = CardZone::remove(&mut player.hand, card) {
                    player.field.push(card);
                }
            }
            self.timestamp += 1;
        }
        Report {
            available_actions: Some(PlayerAvailableActions {
                player: self
                    .players
                    .player_in_turn()
                    .map(|player| player.id)
                    .unwrap_or_default(),
                actions: AvailableActionList::new(),
                instructions: None,
                message_dialog: None,
            }),
            logs: vec![],
            endgame: self.endgame,
            timestamp: self.timestamp,
        }
    }
}

impl Environment {
    pub fn local(&self, viewer: u8) -> LocalEnvironment {
        let players = PlayerList::new(
            self.state
                .players
                .player_in_turn()
                .map(|player| player.id)
                .unwrap_or_default(),
            self.state
                .players
                .iter()
                .map(|player| LocalPlayerState::new(player, viewer)),
        );
        let stack = self.stack.iter().map(|item| item.clone().into()).collect();
        LocalEnvironment {
            player: viewer,
            turn: self.state.turn,
            players,
            phase: self.state.phase,
            stack,
            endgame: self.state.endgame,
            timestamp: self.timestamp,
        }
    }
}
