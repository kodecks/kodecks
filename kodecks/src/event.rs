use bincode::{Decode, Encode};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use strum::Display;

use crate::{
    dsl::script::value::{CustomType, Value},
    player::Zone,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Encode, Decode)]
#[non_exhaustive]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum CardEvent {
    Casted {
        from: Zone,
    },
    Destroyed {
        from: Zone,
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

impl From<CardEvent> for Value {
    fn from(event: CardEvent) -> Value {
        let mut obj = BTreeMap::new();
        match event {
            CardEvent::Casted { from } => {
                let mut from_obj = BTreeMap::new();
                from_obj.insert(
                    "player".into(),
                    Value::Custom(CustomType::Player(from.player)),
                );
                from_obj.insert(
                    "zone".into(),
                    from.kind.to_string().to_ascii_lowercase().into(),
                );
                obj.insert("from".into(), Value::Object(from_obj));
            }
            CardEvent::Destroyed { from, reason } => {
                let mut from_obj = BTreeMap::new();
                from_obj.insert(
                    "player".into(),
                    Value::Custom(CustomType::Player(from.player)),
                );
                from_obj.insert(
                    "zone".into(),
                    from.kind.to_string().to_ascii_lowercase().into(),
                );
                obj.insert("from".into(), Value::Object(from_obj));
                obj.insert(
                    "reason".into(),
                    reason.to_string().to_ascii_lowercase().into(),
                );
            }
            CardEvent::ReturnedToHand { reason } => {
                obj.insert(
                    "reason".into(),
                    reason.to_string().to_ascii_lowercase().into(),
                );
            }
            CardEvent::DealtDamage {
                player,
                amount,
                reason,
            } => {
                obj.insert("player".into(), Value::Custom(CustomType::Player(player)));
                obj.insert("amount".into(), amount.into());
                obj.insert("reason".into(), reason.to_string().into());
            }
            _ => {}
        }
        obj.insert("name".into(), event.as_str().to_string().into());
        Value::Object(obj)
    }
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

    pub fn as_str(&self) -> &'static str {
        match self {
            CardEvent::Casted { .. } => "casted",
            CardEvent::Destroyed { .. } => "destroyed",
            CardEvent::ReturnedToHand { .. } => "returned_to_hand",
            CardEvent::ReturnedToDeck => "returned_to_deck",
            CardEvent::DealtDamage { .. } => "dealt_damage",
            CardEvent::Attacking => "attacking",
            CardEvent::Blocking => "blocking",
            CardEvent::Attacked => "attacked",
            CardEvent::AnyCasted => "any_casted",
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct EventFilter: u16 {
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

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum EventReason {
    Battle,
    Effect,
}
