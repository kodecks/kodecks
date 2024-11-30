use crate::{
    card::Card,
    error::ActionError,
    id::{CardId, ObjectIdCounter},
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
}

impl<T> CardList<T>
where
    T: CardId,
{
    pub fn get_item<I>(&self, id: I) -> Result<&T, ActionError>
    where
        I: CardId,
    {
        self.cards
            .iter()
            .find(|item| item.id() == id.id())
            .ok_or(ActionError::CardNotFound { id: id.id() })
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

    fn remove<I>(&mut self, id: I) -> Option<Card>
    where
        I: CardId,
    {
        let index = self.cards.iter().position(|card| card.id() == id.id())?;
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
