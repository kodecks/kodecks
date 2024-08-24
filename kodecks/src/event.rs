use crate::player::PlayerId;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CardEvent {
    Casted,
    Destroyed,
    DealtDamage { player: PlayerId, amount: u32 },
    Attacking,
    Blocking,
    Attacked,
    AnyCasted,
}

impl CardEvent {
    pub fn filter(&self) -> EventFilter {
        match self {
            CardEvent::Casted => EventFilter::CASTED,
            CardEvent::Destroyed => EventFilter::DESTROYED,
            CardEvent::DealtDamage { .. } => EventFilter::DEALT_DAMAGE,
            CardEvent::Attacking => EventFilter::ATTACKING,
            CardEvent::Blocking => EventFilter::BLOCKING,
            CardEvent::Attacked => EventFilter::ATTACKED,
            CardEvent::AnyCasted => EventFilter::ANY_CASTED,
        }
    }
}

bitflags! {
    #[derive(Clone, Copy, Eq, PartialEq)]
    pub struct EventFilter: u32 {
        const CASTED = 1 << 0;
        const DESTROYED = 1 << 1;
        const DEALT_DAMAGE = 1 << 2;
        const ATTACKING = 1 << 3;
        const BLOCKING = 1 << 4;
        const ATTACKED = 1 << 5;
        const ANY_CASTED = 1 << 6;
    }
}
