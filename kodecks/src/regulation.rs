use crate::deck::DeckList;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Encode, Decode)]
pub struct Regulation {
    pub max_deck_size: u8,
    pub min_deck_size: u8,
    pub max_same_cards: u8,
    pub initial_hand_size: u8,
    pub initial_life: u32,
    pub max_hand_size: u8,
    pub action_timeout: Duration,
    pub phase_timeout: Duration,
    pub max_consecutive_timeouts: u8,
}

impl Default for Regulation {
    fn default() -> Self {
        Self::STANDARD
    }
}

impl Regulation {
    pub const STANDARD: Self = Self {
        max_deck_size: 20,
        min_deck_size: 20,
        max_same_cards: 4,
        initial_hand_size: 4,
        initial_life: 2000,
        max_hand_size: 6,
        action_timeout: Duration::from_secs(30),
        phase_timeout: Duration::from_secs(60),
        max_consecutive_timeouts: 2,
    };

    pub fn verify(&self, deck: &DeckList) -> bool {
        let mut count = HashMap::new();
        for item in &deck.cards {
            let entry = count.entry(item.card.archetype_id).or_insert(0);
            *entry += 1;
            if *entry > self.max_same_cards {
                return false;
            }
        }
        let size = deck.cards.len() as u8;
        size >= self.min_deck_size && size <= self.max_deck_size
    }
}
