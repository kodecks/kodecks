use crate::{config::GameConfig, player::PlayerConfig, scenario::Scenario};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GameProfile {
    pub config: GameConfig,
    pub players: Vec<PlayerConfig>,

    #[serde(skip)]
    pub scenario: Option<Box<dyn Scenario>>,
}
