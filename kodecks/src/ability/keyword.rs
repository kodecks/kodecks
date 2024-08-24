use crate::score::Score;

use super::Ability;
use serde::{Deserialize, Serialize};
use std::{
    mem,
    ops::{Add, Sub},
};
use strum::{Display, EnumString, IntoStaticStr};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
    IntoStaticStr,
    Serialize,
    Deserialize,
    EnumString,
)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum KeywordAbility {
    Toxic,
    Volatile,
}

impl Ability for KeywordAbility {}

impl Score for KeywordAbility {
    fn score(&self) -> i32 {
        match self {
            KeywordAbility::Toxic => 1,
            KeywordAbility::Volatile => -1,
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
