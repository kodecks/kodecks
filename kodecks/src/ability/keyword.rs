use super::Ability;
use crate::score::Score;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{
    mem,
    ops::{Add, Sub},
};
use strum::{Display, EnumIter, EnumString, IntoStaticStr};

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
