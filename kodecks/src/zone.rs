use crate::{dsl::SmallStr, id::CardId};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum::Display;
use tinystr::tinystr;

#[derive(
    Debug, Clone, Copy, Default, Display, PartialEq, Eq, Serialize, Deserialize, Encode, Decode,
)]
#[serde(rename_all = "snake_case")]
pub enum MoveReason {
    #[default]
    Move,
    Draw,
    Fetch,
    Casted,
    Destroyed,
    Discarded,
}

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum ZoneKind {
    Deck,
    Hand,
    Field,
    Colony,
    Graveyard,
}

impl From<ZoneKind> for SmallStr {
    fn from(kind: ZoneKind) -> Self {
        match kind {
            ZoneKind::Deck => tinystr!(32, "deck"),
            ZoneKind::Hand => tinystr!(32, "hand"),
            ZoneKind::Field => tinystr!(32, "field"),
            ZoneKind::Colony => tinystr!(32, "colony"),
            ZoneKind::Graveyard => tinystr!(32, "graveyard"),
        }
    }
}

pub trait CardZone {
    type Item: CardId;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;

    fn iter(&self) -> impl DoubleEndedIterator<Item = &Self::Item>;
    fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Self::Item>;

    fn get<T>(&self, id: T) -> Option<&Self::Item>
    where
        T: CardId,
    {
        self.iter().find(|card| card.id() == id.id())
    }

    fn get_mut<T>(&mut self, id: T) -> Option<&mut Self::Item>
    where
        T: CardId,
    {
        self.iter_mut().find(|card| card.id() == id.id())
    }

    fn contains<T>(&self, id: T) -> bool
    where
        T: CardId,
    {
        self.iter().any(|card| card.id() == id.id())
    }

    fn push(&mut self, card: Self::Item);
    fn remove<T>(&mut self, id: T) -> Option<Self::Item>
    where
        T: CardId;
}
