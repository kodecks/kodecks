use crate::deck::DeckList;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Regulation {
    pub max_deck_size: u8,
    pub min_deck_size: u8,
    pub max_same_cards: u8,
}

impl Default for Regulation {
    fn default() -> Self {
        Self {
            max_deck_size: 20,
            min_deck_size: 20,
            max_same_cards: 4,
        }
    }
}

impl Regulation {
    pub fn verify(&self, deck: &DeckList) -> bool {
        let mut count = HashMap::new();
        for item in &deck.cards {
            let entry = count.entry(item.archetype_id).or_insert(0);
            *entry += 1;
            if *entry > self.max_same_cards {
                return false;
            }
        }
        let size = deck.cards.len() as u8;
        size >= self.min_deck_size && size <= self.max_deck_size
    }
}
