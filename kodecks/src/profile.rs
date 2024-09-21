use crate::{config::GameConfig, player::PlayerConfig, regulation::Regulation, scenario::Scenario};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct GameProfile {
    pub config: GameConfig,
    pub regulation: Regulation,
    pub players: Vec<PlayerConfig>,
    pub bots: Vec<BotConfig>,

    #[serde(skip)]
    pub scenario: Option<Box<dyn Scenario>>,
}

impl fmt::Debug for GameProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameProfile")
            .field("config", &self.config)
            .field("regulation", &self.regulation)
            .field("players", &self.players)
            .field("bots", &self.bots)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub player: u8,
}
