use crate::color::Color;
use crate::error::ActionError;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
#[serde(transparent)]
pub struct ShardList(Vec<(Color, u8)>);

impl ShardList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn get(&self, color: Color) -> u8 {
        self.0
            .iter()
            .find(|(c, _)| *c == color)
            .map(|(_, amount)| *amount)
            .unwrap_or(0)
    }

    pub fn add(&mut self, color: Color, amount: u8) {
        self.0
            .iter_mut()
            .find(|(c, _)| *c == color)
            .map(|(_, current)| *current += amount)
            .unwrap_or_else(|| self.0.push((color, amount)));
    }

    pub fn consume(&mut self, color: Color, amount: u8) -> Result<(), ActionError> {
        let mut insufficient = true;
        if let Some((_, current)) = self.0.iter_mut().find(|(c, _)| *c == color) {
            if *current >= amount {
                insufficient = false;
                *current -= amount;
            } else {
                *current = 0;
            }
        }
        self.0.retain(|(_, amount)| *amount > 0);

        if insufficient {
            Err(ActionError::InsufficientShards { color, amount })
        } else {
            Ok(())
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Color, u8)> + '_ {
        self.0.iter().copied()
    }

    pub fn len(&self) -> usize {
        self.0.iter().map(|(_, amount)| *amount as usize).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
