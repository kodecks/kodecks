use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    action::PlayerAvailableActions,
    card::Card,
    dsl::script::{
        error::Error,
        exp::ExpParams,
        value::{Constant, Value},
    },
    error::ActionError,
    id::{CardId, ObjectId},
    log::GameLog,
    phase::Phase,
    player::{Player, PlayerList, PlayerScore, Zone},
    profile::DebugConfig,
    regulation::Regulation,
    score::Score,
};

use super::{EndgameState, LocalEnvironment};

#[derive(Clone)]
pub struct GameState {
    pub regulation: Regulation,
    pub debug: DebugConfig,
    pub turn: u16,
    pub phase: Phase,
    pub players: PlayerList<Player>,
    pub endgame: EndgameState,
}

impl GameState {
    pub fn find_card<T>(&self, card: T) -> Result<&Card, ActionError>
    where
        T: CardId + Copy,
    {
        self.players
            .iter()
            .filter_map(|player| player.find_card(card))
            .next()
            .ok_or(ActionError::CardNotFound { id: card.id() })
    }

    pub fn find_card_mut(&mut self, card: ObjectId) -> Result<&mut Card, ActionError> {
        self.players
            .iter_mut()
            .filter_map(|player| player.find_card_mut(card))
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

    pub fn players(&self) -> &PlayerList<Player> {
        &self.players
    }

    pub fn get_var(&self, name: &str) -> Option<Value> {
        match name {
            "$turn" => Some(self.turn.into()),
            "$phase" => Some(Constant::String(self.phase.into()).into()),
            "$player_in_turn" => Some(self.players.player_in_turn().ok()?.id.into()),
            "$players" => Some(Value::Array(
                self.players.iter().map(|p| p.id.into()).collect(),
            )),
            _ => None,
        }
    }

    pub fn invoke(
        &self,
        name: &str,
        args: Vec<Value>,
        _params: &ExpParams,
        input: &Value,
    ) -> Result<Vec<Value>, Error> {
        match name {
            "debug" => {
                let args = args
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                debug!("{args}");
                Ok(vec![input.clone()])
            }
            _ => Err(Error::UndefinedFilter),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct LocalGameState {
    pub env: LocalEnvironment,
    pub logs: Vec<GameLog>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available_actions: Option<PlayerAvailableActions>,
}

#[derive(Debug, Clone)]
pub struct GameStateScore {
    pub players: Vec<PlayerScore>,
    pub endgame: EndgameState,
}

impl Score for GameState {
    type Output = GameStateScore;

    fn score(&self) -> GameStateScore {
        GameStateScore {
            players: self.players.iter().map(Player::score).collect(),
            endgame: self.endgame,
        }
    }
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
                        colony: vec![],
                        deck: 100,
                        graveyard: vec![],
                        limbo: vec![],
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
