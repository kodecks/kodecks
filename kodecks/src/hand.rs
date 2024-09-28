use crate::{
    card::Card,
    id::{CardId, ObjectId, TimedObjectId},
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct HandItem<T: CardId> {
    pub card: T,
    pub cost_delta: i8,
}

impl<T> CardId for HandItem<T>
where
    T: CardId,
{
    fn id(&self) -> ObjectId {
        self.card.id()
    }

    fn timed_id(&self) -> TimedObjectId {
        self.card.timed_id()
    }
}

impl AsRef<Card> for HandItem<Card> {
    fn as_ref(&self) -> &Card {
        &self.card
    }
}

impl AsMut<Card> for HandItem<Card> {
    fn as_mut(&mut self) -> &mut Card {
        &mut self.card
    }
}

impl From<Card> for HandItem<Card> {
    fn from(card: Card) -> Self {
        Self {
            card,
            cost_delta: 0,
        }
    }
}

impl From<HandItem<Card>> for Card {
    fn from(item: HandItem<Card>) -> Self {
        item.card
    }
}
