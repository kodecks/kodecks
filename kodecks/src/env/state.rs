use serde_tuple::{Deserialize_tuple, Serialize_tuple};

use crate::{
    action::PlayerAvailableActions,
    card::Card,
    error::Error,
    id::ObjectId,
    log::LogAction,
    phase::Phase,
    player::{PlayerList, PlayerState, PlayerZone},
    profile::DebugConfig,
    regulation::Regulation,
};

use super::LocalEnvironment;

#[derive(Clone)]
pub struct GameState {
    pub regulation: Regulation,
    pub debug: DebugConfig,
    pub turn: u32,
    pub phase: Phase,
    pub players: PlayerList<PlayerState>,
}

impl GameState {
    pub fn find_card(&self, card: ObjectId) -> Result<&Card, Error> {
        self.players
            .iter()
            .filter_map(|player| player.find_card(card))
            .next()
            .ok_or(Error::CardNotFound { id: card })
    }

    pub fn find_card_mut(&mut self, card: ObjectId) -> Result<&mut Card, Error> {
        self.players
            .iter_mut()
            .filter_map(|player| player.find_card_mut(card))
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

    pub fn players(&self) -> &PlayerList<PlayerState> {
        &self.players
    }
}

#[derive(Debug, Clone, Serialize_tuple, Deserialize_tuple)]
pub struct LocalGameState {
    pub env: LocalEnvironment,
    pub logs: Vec<LogAction>,
    pub available_actions: Option<PlayerAvailableActions>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        color::Color,
        env::GameCondition,
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
                game_condition: GameCondition::Progress,
            },
            logs: vec![LogAction::LifeChanged { player, life: 100 }],
            available_actions: Some(PlayerAvailableActions::new(player)),
        };

        let serialized = serde_json::to_string(&game_state).unwrap();
        serde_json::from_str::<LocalGameState>(&serialized).unwrap();
    }
}
