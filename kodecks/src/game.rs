use crate::{
    action::{Action, PlayerAvailableActions},
    card::Catalog,
    env::{Environment, GameCondition, LocalEnvironment},
    log::LogAction,
    player::PlayerId,
    profile::GameProfile,
    scenario::Scenario,
};
use serde_tuple::{Deserialize_tuple, Serialize_tuple};
use std::sync::Arc;

pub struct Game {
    env: Arc<Environment>,
    scenario: Option<Box<dyn Scenario>>,
}

impl Game {
    pub fn new(profile: GameProfile, catalog: &'static Catalog) -> Game {
        Game {
            env: Arc::new(Environment::new(profile, catalog)),
            scenario: None,
        }
    }

    pub fn tick(&mut self, player: PlayerId, action: Option<Action>) -> Report {
        let mut report = Arc::make_mut(&mut self.env).process(player, action);
        if let Some(scenario) = &mut self.scenario {
            report.available_actions =
                scenario.override_actions(&self.env, report.available_actions);
        }
        report
    }

    pub fn env(&self) -> &Arc<Environment> {
        &self.env
    }
}

#[derive(Debug, Clone)]
pub struct Report {
    pub available_actions: Option<PlayerAvailableActions>,
    pub logs: Vec<LogAction>,
    pub condition: GameCondition,
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
        id::ObjectIdCounter,
        phase::Phase,
        player::{LocalPlayerState, PlayerList},
        shard::ShardList,
        stack::LocalStackItem,
    };

    #[test]
    fn test_serialize_local_game_state() {
        let player = PlayerId::new("player1");
        let mut counter = ObjectIdCounter::default();
        let mut shards = ShardList::new();
        shards.add(Color::RED, 10);
        let game_state = LocalGameState {
            env: LocalEnvironment {
                player,
                turn: 1,
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
