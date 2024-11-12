use super::Ability;
use crate::{
    dsl::script::value::{Constant, Value},
    score::Score,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    mem,
    ops::{Add, Sub},
};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};
use tinystr::tinystr;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
    IntoStaticStr,
    Serialize,
    Deserialize,
    EnumString,
    EnumIter,
    Encode,
    Decode,
)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum KeywordAbility {
    Toxic,
    Volatile,
    Stealth,
    Devour,
    Piercing,
}

impl Ability for KeywordAbility {}

impl Score for KeywordAbility {
    fn score(&self) -> i32 {
        match self {
            KeywordAbility::Toxic => 1,
            KeywordAbility::Volatile => -1,
            KeywordAbility::Stealth => 1,
            KeywordAbility::Devour => 1,
            KeywordAbility::Piercing => 1,
        }
    }
}

impl Add for KeywordAbility {
    type Output = (Option<Self>, Option<Self>);

    fn add(self, rhs: Self) -> Self::Output {
        if mem::discriminant(&self) == mem::discriminant(&rhs) {
            (Some(self), None)
        } else {
            (Some(self), Some(rhs))
        }
    }
}

impl Sub for KeywordAbility {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Option<Self> {
        if mem::discriminant(&self) == mem::discriminant(&rhs) {
            None
        } else {
            Some(self)
        }
    }
}

impl From<KeywordAbility> for Value {
    fn from(ability: KeywordAbility) -> Value {
        let name = match ability {
            KeywordAbility::Toxic => tinystr!(32, "toxic"),
            KeywordAbility::Volatile => tinystr!(32, "volatile"),
            KeywordAbility::Stealth => tinystr!(32, "stealth"),
            KeywordAbility::Devour => tinystr!(32, "devour"),
            KeywordAbility::Piercing => tinystr!(32, "piercing"),
        };
        let mut obj = BTreeMap::new();
        obj.insert(tinystr!(32, "name"), Constant::String(name).into());
        Value::Object(obj)
    }
}
