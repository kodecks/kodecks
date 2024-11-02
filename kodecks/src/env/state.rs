use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{
    action::PlayerAvailableActions,
    card::Card,
    error::ActionError,
    id::ObjectId,
    log::GameLog,
    phase::Phase,
    player::{Player, PlayerList, PlayerZone},
    profile::DebugConfig,
    regulation::Regulation,
};

use super::LocalEnvironment;

#[derive(Clone)]
pub struct GameState {
    pub regulation: Regulation,
    pub debug: DebugConfig,
    pub turn: u16,
    pub phase: Phase,
    pub players: PlayerList<Player>,
}

impl GameState {
    pub fn find_card(&self, card: ObjectId) -> Result<&Card, ActionError> {
        self.players
            .iter()
            .filter_map(|player| player.find_card(card))
            .next()
            .ok_or(ActionError::CardNotFound { id: card })
    }

    pub fn find_card_mut(&mut self, card: ObjectId) -> Result<&mut Card, ActionError> {
        self.players
            .iter_mut()
            .filter_map(|player| player.find_card_mut(card))
            .next()
            .ok_or(ActionError::CardNotFound { id: card })
    }

    pub fn find_zone(&self, card: ObjectId) -> Result<PlayerZone, ActionError> {
        for player in self.players.iter() {
            if let Some(zone) = player.find_zone(card) {
                return Ok(PlayerZone {
                    player: player.id,
                    zone,
                });
            }
        }
        Err(ActionError::CardNotFound { id: card })
    }

    pub fn players(&self) -> &PlayerList<Player> {
        &self.players
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct LocalGameState {
    pub env: LocalEnvironment,
    pub logs: Vec<GameLog>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available_actions: Option<PlayerAvailableActions>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        color::Color,
        env::EndgameState,
        id::ObjectIdCounter,
        phase::Phase,
        player::{LocalPlayerState, PlayerList},
        shard::ShardList,
        stack::LocalStackItem,
    };

    #[test]
    fn test_serialize_local_game_state() {
        let player = 1;
        let mut counter = ObjectIdCounter::default();
        let mut shards = ShardList::new();
        shards.add(Color::RED, 10);
        let game_state = LocalGameState {
            env: LocalEnvironment {
                player,
                turn: 1,
                timestamp: 0,
                players: PlayerList::new(
                    player,
                    vec![LocalPlayerState {
                        id: player,
                        hand: vec![],
                        field: vec![],
                        deck: 100,
                        graveyard: vec![],
                        shards,
                        stats: Default::default(),
                    }],
                ),
                phase: Phase::Main,
                stack: vec![LocalStackItem {
                    source: counter.allocate(None),
                    id: "1".to_string(),
                }]
                .into_iter()
                .collect(),
                endgame: EndgameState::InProgress,
            },
            logs: vec![GameLog::LifeChanged { player, life: 100 }],
            available_actions: Some(PlayerAvailableActions::new(player)),
        };

        let serialized = serde_json::to_string(&game_state).unwrap();
        serde_json::from_str::<LocalGameState>(&serialized).unwrap();

        let config = bincode::config::standard();
        let serialized = bincode::encode_to_vec(&game_state, bincode::config::standard()).unwrap();
        bincode::decode_from_slice::<LocalGameState, _>(&serialized, config).unwrap();
    }
}
