use bincode::{Decode, Encode};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::player::PlayerZone;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Encode, Decode)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum CardEvent {
    Casted {
        from: PlayerZone,
    },
    Destroyed {
        from: PlayerZone,
        reason: EventReason,
    },
    ReturnedToHand {
        reason: EventReason,
    },
    ReturnedToDeck,
    DealtDamage {
        player: u8,
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
            CardEvent::Casted { .. } => EventFilter::CASTED,
            CardEvent::Destroyed { .. } => EventFilter::DESTROYED,
            CardEvent::ReturnedToHand { .. } => EventFilter::RETURNED_TO_HAND,
            CardEvent::ReturnedToDeck => EventFilter::RETURNED_TO_DECK,
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
        const RETURNED_TO_DECK = 1 << 3;
        const DEALT_DAMAGE = 1 << 4;
        const ATTACKING = 1 << 5;
        const BLOCKING = 1 << 6;
        const ATTACKED = 1 << 7;
        const ANY_CASTED = 1 << 8;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum EventReason {
    Battle,
    Effect,
}
