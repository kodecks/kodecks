use crate::{
    card::{safe_name, ArchetypeId, Card, Catalog},
    id::{ObjectId, ObjectIdCounter},
    sequence::CardSequence,
    zone::CardZone,
};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter;

#[derive(Debug, Default)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn shuffle(&mut self, counter: &mut ObjectIdCounter, rng: &mut impl rand::Rng) {
        self.cards.shuffle(rng);
        for card in &mut self.cards {
            card.renew_id(counter);
        }
        self.cards.shuffle(rng);
    }
}

impl CardZone for Deck {
    type Item = Card;

    fn len(&self) -> usize {
        self.cards.len()
    }

    fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = &Card> {
        self.cards.iter()
    }

    fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Card> {
        self.cards.iter_mut()
    }

    fn push(&mut self, card: Card) {
        self.cards.push(card);
    }

    fn remove(&mut self, id: ObjectId) -> Option<Card> {
        let index = self.cards.iter().position(|card| card.id() == id)?;
        Some(self.cards.remove(index))
    }

    fn duplicate(&self) -> Self
    where
        Self: Sized,
    {
        Self {
            cards: self.cards.iter().map(|card| card.duplicate()).collect(),
        }
    }
}

impl CardSequence for Deck {
    fn remove_top(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    fn add_top(&mut self, card: Card) {
        self.cards.push(card);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeckList {
    pub cards: Vec<DeckItem>,
}

impl fmt::Display for DeckList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.cards {
            writeln!(f, "{}", item)?;
        }
        Ok(())
    }
}

impl DeckList {
    pub fn parse(s: &str, catalog: &'static Catalog) -> Option<Self> {
        let cards = s
            .lines()
            .map(|s| s.trim())
            .flat_map(|line| DeckItem::parse(line, catalog))
            .collect();
        Some(Self { cards })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeckItem {
    pub archetype_id: ArchetypeId,
    pub base_id: Option<ObjectId>,
}

impl DeckItem {
    pub fn parse(s: &str, catalog: &'static Catalog) -> Vec<Self> {
        let (name, count) = match s.rsplit_once(' ') {
            Some((name, count)) => {
                if let Ok(count) = count.parse() {
                    (name, count)
                } else {
                    (s, 1)
                }
            }
            _ => (s, 1),
        };

        if name.is_empty() {
            return vec![];
        }
        let name = safe_name(name).unwrap();
        let card = &catalog[name.as_str()];
        iter::repeat(Self {
            archetype_id: card.id,
            base_id: None,
        })
        .take(count)
        .collect()
    }
}

impl fmt::Display for DeckItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.archetype_id)
    }
}
