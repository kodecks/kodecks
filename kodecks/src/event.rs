use crate::player::PlayerId;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CardEvent {
    Casted,
    Destroyed {
        reason: EventReason,
    },
    ReturnedToHand {
        reason: EventReason,
    },
    DealtDamage {
        player: PlayerId,
        amount: u32,
        reason: EventReason,
    },
    Attacking,
    Blocking,
    Attacked,
    AnyCasted,
}

impl CardEvent {
    pub fn filter(&self) -> EventFilter {
        match self {
            CardEvent::Casted => EventFilter::CASTED,
            CardEvent::Destroyed { .. } => EventFilter::DESTROYED,
            CardEvent::ReturnedToHand { .. } => EventFilter::RETURNED_TO_HAND,
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
        const RETURNED_TO_HAND = 1 << 2;
        const DEALT_DAMAGE = 1 << 3;
        const ATTACKING = 1 << 4;
        const BLOCKING = 1 << 5;
        const ATTACKED = 1 << 6;
        const ANY_CASTED = 1 << 7;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventReason {
    Battle,
    Effect,
}
