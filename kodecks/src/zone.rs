use crate::id::{CardId, ObjectId};
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

    fn get(&self, id: ObjectId) -> Option<&Self::Item> {
        self.iter().find(|card| card.id() == id)
    }

    fn get_mut(&mut self, id: ObjectId) -> Option<&mut Self::Item> {
        self.iter_mut().find(|card| card.id() == id)
    }

    fn contains(&self, id: ObjectId) -> bool {
        self.iter().any(|card| card.id() == id)
    }

    fn push(&mut self, card: Self::Item);
    fn remove(&mut self, id: ObjectId) -> Option<Self::Item>;
}
