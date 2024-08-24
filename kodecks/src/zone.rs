use crate::id::{CardId, ObjectId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveReason {
    #[default]
    Move,
    Draw,
    Casted,
    Destroyed,
    Discarded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Zone {
    Deck,
    Hand,
    Queue,
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

    fn duplicate(&self) -> Self
    where
        Self: Sized;
}
