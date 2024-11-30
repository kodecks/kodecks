use crate::{card::Card, id::CardId, zone::CardZone};

#[derive(Debug, Clone)]
pub struct CardSlot<T> {
    cards: Vec<Option<T>>,
}

impl<T> CardSlot<T> {
    pub fn new(len: usize) -> Self {
        Self {
            cards: (0..len).into_iter().map(|_| None).collect(),
        }
    }
}

impl<T> CardZone for CardSlot<T>
where
    T: CardId + AsRef<Card> + AsMut<Card> + From<Card> + Into<Card>,
{
    type Item = Card;

    fn len(&self) -> usize {
        self.cards.iter().filter(|card| card.is_some()).count()
    }

    fn is_empty(&self) -> bool {
        self.cards.iter().all(|card| card.is_none())
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = &Card> {
        self.cards
            .iter()
            .filter_map(|card| card.as_ref().map(|card| card.as_ref()))
    }

    fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Card> {
        self.cards
            .iter_mut()
            .filter_map(|card| card.as_mut().map(|card| card.as_mut()))
    }

    fn push(&mut self, card: Card) {
        if let Some(slot) = self.cards.iter_mut().find(|card| card.is_none()) {
            *slot = Some(card.into());
        }
    }

    fn remove<I>(&mut self, id: I) -> Option<Card>
    where
        I: CardId,
    {
        let index = self
            .cards
            .iter()
            .position(|card| card.as_ref().map_or(false, |card| card.id() == id.id()))?;
        self.cards[index].take().map(|card| card.into())
    }

    fn get<I>(&self, id: I) -> Option<&Self::Item>
    where
        I: CardId,
    {
        self.cards.iter().find_map(|card| {
            card.as_ref()
                .map(|card| card.as_ref())
                .filter(|card| card.id() == id.id())
        })
    }
}
