use crate::id::CardId;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(
    Debug, Clone, Copy, Default, Display, PartialEq, Eq, Serialize, Deserialize, Encode, Decode,
)]
#[serde(rename_all = "snake_case")]
pub enum MoveReason {
    #[default]
    Move,
    Draw,
    Casted,
    Destroyed,
    Discarded,
}

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum Zone {
    Deck,
    Hand,
    Field,
    Graveyard,
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
