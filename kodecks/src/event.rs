use bincode::{Decode, Encode};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use strum::Display;
use tinystr::tinystr;

use crate::{
    dsl::{
        script::value::{Constant, CustomType, Value},
        SmallStr,
    },
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
                    tinystr!(32, "player"),
                    Value::Custom(CustomType::Player(from.player)),
                );
                from_obj.insert(
                    tinystr!(32, "zone"),
                    Constant::String(from.kind.into()).into(),
                );
                obj.insert(tinystr!(32, "from"), Value::Object(from_obj));
            }
            CardEvent::Destroyed { from, reason } => {
                let mut from_obj = BTreeMap::new();
                from_obj.insert(
                    tinystr!(32, "player"),
                    Value::Custom(CustomType::Player(from.player)),
                );
                from_obj.insert(
                    tinystr!(32, "zone"),
                    Constant::String(from.kind.into()).into(),
                );
                obj.insert(tinystr!(32, "from"), Value::Object(from_obj));
                obj.insert(
                    tinystr!(32, "reason"),
                    Constant::String(reason.into()).into(),
                );
            }
            CardEvent::ReturnedToHand { reason } => {
                obj.insert(
                    tinystr!(32, "reason"),
                    Constant::String(reason.into()).into(),
                );
            }
            CardEvent::DealtDamage {
                player,
                amount,
                reason,
            } => {
                obj.insert(
                    tinystr!(32, "player"),
                    Value::Custom(CustomType::Player(player)),
                );
                obj.insert(tinystr!(32, "amount"), amount.into());
                obj.insert(
                    tinystr!(32, "reason"),
                    Constant::String(reason.into()).into(),
                );
            }
            _ => {}
        }
        obj.insert(tinystr!(32, "name"), Constant::String(event.into()).into());
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
}

impl From<CardEvent> for SmallStr {
    fn from(event: CardEvent) -> SmallStr {
        match event {
            CardEvent::Casted { .. } => tinystr!(32, "casted"),
            CardEvent::Destroyed { .. } => tinystr!(32, "destroyed"),
            CardEvent::ReturnedToHand { .. } => tinystr!(32, "returned_to_hand"),
            CardEvent::ReturnedToDeck => tinystr!(32, "returned_to_deck"),
            CardEvent::DealtDamage { .. } => tinystr!(32, "dealt_damage"),
            CardEvent::Attacking => tinystr!(32, "attacking"),
            CardEvent::Blocking => tinystr!(32, "blocking"),
            CardEvent::Attacked => tinystr!(32, "attacked"),
            CardEvent::AnyCasted => tinystr!(32, "any_casted"),
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

impl From<EventReason> for SmallStr {
    fn from(reason: EventReason) -> SmallStr {
        match reason {
            EventReason::Battle => tinystr!(32, "battle"),
            EventReason::Effect => tinystr!(32, "effect"),
        }
    }
}
