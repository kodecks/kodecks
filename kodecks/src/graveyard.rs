use crate::{card::Card, id::ObjectId, sequence::CardSequence, zone::CardZone};

#[derive(Debug, Default)]
pub struct Graveyard {
    cards: Vec<Card>,
}

impl CardZone for Graveyard {
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

impl CardSequence for Graveyard {
    fn remove_top(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    fn add_top(&mut self, card: Card) {
        self.cards.push(card);
    }
}
