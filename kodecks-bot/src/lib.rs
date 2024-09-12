#![forbid(unsafe_code)]

use bitflags::bitflags;
use kodecks::{
    action::{Action, PlayerAvailableActions},
    env::Environment,
    
};
use rand::rngs::SmallRng;
use score::ComputedScore;
use std::sync::Arc;

mod battle;
mod cast;
mod default;
mod score;
mod select;
mod simple;

pub use default::DefaultBot;
pub use simple::SimpleBot;

#[derive(Clone)]
pub struct BotContext {
    pub rng: SmallRng,
    pub flags: BotFlags,
    pub player: u8,
    pub env: Arc<Environment>,
}

bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct BotFlags: u8 {
        const BATTLE_ONLY = 0b00000001;
        const ALWAYS_BATTLE = 0b00000010;
    }
}

pub trait Bot {
    fn compute(
        &mut self,
        env: Arc<Environment>,
        actions: &PlayerAvailableActions,
    ) -> Vec<(Action, ComputedScore)>;

    fn compute_best_action(
        &mut self,
        env: Arc<Environment>,
        actions: &PlayerAvailableActions,
    ) -> Option<Action> {
        self.compute(env, actions)
            .into_iter()
            .max_by_key(|(_, score)| *score)
            .map(|(action, _)| action)
    }
}
