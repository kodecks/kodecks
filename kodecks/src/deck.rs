use crate::{
    card::{safe_name, CardEntry},
    catalog::Catalog,
    id::ObjectId,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, Hash)]
pub struct DeckList {
    pub id: String,
    pub name: String,
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
    pub fn parse(s: &str, catalog: &Catalog) -> Option<Self> {
        let mut name = String::new();
        let mut cards = Vec::new();
        for line in s.lines().map(|s| s.trim()) {
            if line.starts_with('#') {
                name = line.trim_start_matches('#').trim().to_string();
            } else {
                cards.extend(DeckItem::parse(line, catalog));
            }
        }
        let id = nanoid::nanoid!();
        Some(Self { id, name, cards })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, Hash)]
pub struct DeckItem {
    #[serde(flatten)]
    pub card: CardEntry,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_id: Option<ObjectId>,
}

impl DeckItem {
    pub fn parse(s: &str, catalog: &Catalog) -> Vec<Self> {
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
            card: CardEntry {
                archetype_id: card.id,
                style: 0,
            },
            base_id: None,
        })
        .take(count)
        .collect()
    }
}

impl fmt::Display for DeckItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.card.archetype_id)
    }
}
