use super::Ability;
use crate::score::Score;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{
    mem,
    ops::{Add, Sub},
};

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
