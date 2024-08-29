use crate::{
    card::Card,
    error::Error,
    id::{CardId, ObjectId, TimedObjectId},
    sequence::CardSequence,
    zone::CardZone,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct Hand {
    cards: Vec<HandItem<Card>>,
}

impl Hand {
    pub fn get_item(&self, id: ObjectId) -> Result<&HandItem<Card>, Error> {
        self.cards
            .iter()
            .find(|item| item.id() == id)
            .ok_or(Error::CardNotFound { id })
    }

    pub fn items(&self) -> impl Iterator<Item = &HandItem<Card>> {
        self.cards.iter()
    }

    pub fn items_mut(&mut self) -> impl Iterator<Item = &mut HandItem<Card>> {
        self.cards.iter_mut()
    }
}

impl CardZone for Hand {
    type Item = Card;

    fn len(&self) -> usize {
        self.cards.len()
    }

    fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = &Card> {
        self.cards.iter().map(|item| &item.card)
    }

    fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Card> {
        self.cards.iter_mut().map(|item| &mut item.card)
    }

    fn push(&mut self, card: Card) {
        self.cards.push(HandItem {
            card,
            cost_delta: 0,
        });
    }

    fn remove(&mut self, id: ObjectId) -> Option<Card> {
        let index = self.cards.iter().position(|item| item.id() == id)?;
        Some(self.cards.remove(index).card)
    }

    fn duplicate(&self) -> Self
    where
        Self: Sized,
    {
        Self {
            cards: self
                .cards
                .iter()
                .map(|item| HandItem {
                    card: item.card.duplicate(),
                    cost_delta: item.cost_delta,
                })
                .collect(),
        }
    }
}

impl CardSequence for Hand {
    fn remove_top(&mut self) -> Option<Card> {
        self.cards.pop().map(|item| item.card)
    }

    fn add_top(&mut self, card: Card) {
        self.cards.push(HandItem {
            card,
            cost_delta: 0,
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
