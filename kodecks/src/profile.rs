use crate::{player::PlayerConfig, regulation::Regulation};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GameProfile {
    pub regulation: Regulation,
    pub debug: DebugConfig,
    pub players: Vec<PlayerConfig>,
    pub bots: Vec<BotConfig>,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct DebugConfig {
    pub rng_seed: Option<u64>,
    pub no_deck_shuffle: bool,
    pub no_player_shuffle: bool,
    pub flags: DebugFlags,
}

bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DebugFlags: u8 {
        const DEBUG_COMMAND = 0b00000001;
        const IGNORE_COST = 0b00000010;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    pub player: u8,
}
