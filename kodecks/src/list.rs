use crate::{
    card::Card,
    error::Error,
    id::{CardId, ObjectId, ObjectIdCounter},
    sequence::CardSequence,
    zone::CardZone,
};
use rand::seq::SliceRandom;
#[derive(Debug, Clone)]
pub struct CardList<T> {
    cards: Vec<T>,
}

impl<T> CardList<T> {
    pub fn new() -> Self {
        Self { cards: Vec::new() }
    }

    pub fn items(&self) -> impl Iterator<Item = &T> {
        self.cards.iter()
    }

    pub fn items_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.cards.iter_mut()
    }
}

impl<T> CardList<T>
where
    T: CardId,
{
    pub fn get_item(&self, id: ObjectId) -> Result<&T, Error> {
        self.cards
            .iter()
            .find(|item| item.id() == id)
            .ok_or(Error::CardNotFound { id })
    }
}

impl<T> Default for CardList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CardZone for CardList<T>
where
    T: CardId + AsRef<Card> + AsMut<Card> + From<Card> + Into<Card>,
{
    type Item = Card;

    fn len(&self) -> usize {
        self.cards.len()
    }

    fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = &Card> {
        self.cards.iter().map(|card| card.as_ref())
    }

    fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Card> {
        self.cards.iter_mut().map(|card| card.as_mut())
    }

    fn push(&mut self, card: Card) {
        self.cards.push(card.into());
    }

    fn remove(&mut self, id: ObjectId) -> Option<Card> {
        let index = self.cards.iter().position(|card| card.id() == id)?;
        Some(self.cards.remove(index).into())
    }
}

impl<T> CardSequence for CardList<T>
where
    T: CardId + AsRef<Card> + AsMut<Card> + From<Card> + Into<Card>,
{
    fn remove_top(&mut self) -> Option<Card> {
        self.cards.pop().map(|card| card.into())
    }

    fn add_top(&mut self, card: Card) {
        self.cards.push(card.into());
    }
}

impl CardList<Card> {
    pub fn shuffle(&mut self, counter: &mut ObjectIdCounter, rng: &mut impl rand::Rng) {
        self.cards.shuffle(rng);
        for card in &mut self.cards {
            card.renew_id(counter);
        }
        self.cards.shuffle(rng);
    }
}