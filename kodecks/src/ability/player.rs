use crate::score::Score;

use super::Ability;
use std::{
    mem,
    ops::{Add, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum PlayerAbility {
    Propagate(i32),
    Draw,
}

impl Ability for PlayerAbility {}

impl Score for PlayerAbility {
    type Output = i32;

    fn score(&self) -> i32 {
        match self {
            PlayerAbility::Propagate(n) => *n,
            PlayerAbility::Draw => 1,
        }
    }
}

impl Add for PlayerAbility {
    type Output = (Option<Self>, Option<Self>);

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (PlayerAbility::Propagate(a), PlayerAbility::Propagate(b)) => {
                let sum = a + b;
                (
                    Some(PlayerAbility::Propagate(sum)).filter(|_| sum != 0),
                    None,
                )
            }
            _ => {
                if mem::discriminant(&self) == mem::discriminant(&rhs) {
                    (Some(self), None)
                } else {
                    (Some(self), Some(rhs))
                }
            }
        }
    }
}

impl Sub for PlayerAbility {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Option<Self> {
        if mem::discriminant(&self) == mem::discriminant(&rhs) {
            None
        } else {
            Some(self)
        }
    }
}
