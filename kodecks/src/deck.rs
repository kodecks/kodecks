use crate::{
    card::{safe_name, ArchetypeId, Catalog},
    id::ObjectId,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
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
