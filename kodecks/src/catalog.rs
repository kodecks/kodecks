use crate::archetype::{ArchetypeId, CardArchetype};
use std::{
    collections::HashMap,
    ops::Index,
    sync::{Arc, LazyLock},
};

pub type CardList = [fn() -> &'static CardArchetype];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    pub sets: Vec<CardSet>,
}

impl Catalog {
    pub fn new(cards: &CardList) -> Self {
        Self {
            sets: vec![CardSet::new(cards)],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<CardArchetype>> {
        self.sets.iter().flat_map(|set| set.iter())
    }

    pub fn get<S>(&self, id: S) -> Option<&Arc<CardArchetype>>
    where
        S: AsRef<str>,
    {
        self.sets.iter().find_map(|set| set.get(id.as_ref()))
    }

    pub fn contains<S>(&self, id: S) -> bool
    where
        S: AsRef<str>,
    {
        let id = id.as_ref();
        self.sets.iter().any(|set| set.contains(id))
    }
}

impl Index<&str> for Catalog {
    type Output = Arc<CardArchetype>;

    fn index(&self, id: &str) -> &Self::Output {
        static NONE: LazyLock<Arc<CardArchetype>> =
            LazyLock::new(|| Arc::new(CardArchetype::default()));
        self.get(id).unwrap_or(&NONE)
    }
}

impl Index<ArchetypeId> for Catalog {
    type Output = Arc<CardArchetype>;

    fn index(&self, short_id: ArchetypeId) -> &Self::Output {
        &self[short_id.as_str()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSet {
    map: HashMap<String, usize>,
    list: Vec<Arc<CardArchetype>>,
}

impl CardSet {
    pub fn new(cards: &CardList) -> Self {
        let mut list = cards
            .iter()
            .map(|archetype| Arc::new(archetype().clone()))
            .collect::<Vec<_>>();
        list.sort();
        let map = list
            .iter()
            .enumerate()
            .flat_map(|(i, card)| {
                [
                    (card.id.as_str().to_string(), i),
                    (card.safe_name.to_string(), i),
                ]
            })
            .collect();
        Self { map, list }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<CardArchetype>> {
        self.list.iter()
    }

    pub fn get<S>(&self, id: S) -> Option<&Arc<CardArchetype>>
    where
        S: AsRef<str>,
    {
        self.map.get(id.as_ref()).map(|&i| &self.list[i])
    }

    pub fn contains<S>(&self, id: S) -> bool
    where
        S: AsRef<str>,
    {
        self.map.contains_key(id.as_ref())
    }
}
