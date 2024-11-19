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
use tinystr::tinystr;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Encode, Decode,
)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum AnonymousAbility {
    Defender,
}

impl Ability for AnonymousAbility {}

impl Score for AnonymousAbility {
    type Output = i32;

    fn score(&self) -> i32 {
        match self {
            AnonymousAbility::Defender => -1,
        }
    }
}

impl Add for AnonymousAbility {
    type Output = (Option<Self>, Option<Self>);

    fn add(self, rhs: Self) -> Self::Output {
        if mem::discriminant(&self) == mem::discriminant(&rhs) {
            (Some(self), None)
        } else {
            (Some(self), Some(rhs))
        }
    }
}

impl Sub for AnonymousAbility {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Option<Self> {
        if mem::discriminant(&self) == mem::discriminant(&rhs) {
            None
        } else {
            Some(self)
        }
    }
}

impl From<AnonymousAbility> for Value {
    fn from(ability: AnonymousAbility) -> Value {
        let name = match ability {
            AnonymousAbility::Defender => tinystr!(32, "defender"),
        };
        let mut obj = BTreeMap::new();
        obj.insert(tinystr!(32, "name"), Constant::String(name).into());
        Value::Object(obj)
    }
}
