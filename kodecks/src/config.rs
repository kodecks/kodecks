use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameConfig {
    pub rng_seed: Option<u64>,
    pub initial_hand_size: u8,
    pub initial_life: u32,
    pub max_hand_size: u8,
    pub no_deck_shuffle: bool,
    pub no_player_shuffle: bool,
    pub debug: DebugFlags,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            rng_seed: None,
            initial_hand_size: 4,
            initial_life: 2000,
            max_hand_size: 6,
            no_deck_shuffle: false,
            no_player_shuffle: false,
            debug: DebugFlags::empty(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DebugFlags: u8 {
        const DEBUG_COMMAND = 0b00000001;
        const IGNORE_COST = 0b00000010;
    }
}
