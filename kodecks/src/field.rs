use crate::{
    card::Card,
    id::{CardId, ObjectId},
    score::Score,
    sequence::CardSequence,
    zone::CardZone,
};
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Default)]
pub struct Field {
    cards: Vec<FieldItem<Card>>,
}

impl Field {
    pub fn active_cards(&self) -> impl Iterator<Item = &Card> {
        self.cards.iter().filter_map(|field_card| {
            if field_card.state == FieldState::Active {
                Some(&field_card.card)
            } else {
                None
            }
        })
    }

    pub fn attacking_cards(&self) -> impl Iterator<Item = &Card> {
        self.cards.iter().filter_map(|field_card| {
            if let Some(FieldBattleState::Attacking) = field_card.battle {
                Some(&field_card.card)
            } else {
                None
            }
        })
    }

    pub fn find_blocker(&self, attacker: ObjectId) -> Option<&Card> {
        self.cards
            .iter()
            .find_map(|field_card| match field_card.battle {
                Some(FieldBattleState::Blocking { attacker: id }) if id == attacker => {
                    Some(&field_card.card)
                }
                _ => None,
            })
    }

    pub fn set_card_state(&mut self, id: ObjectId, state: FieldState) {
        if let Some(field_card) = self
            .cards
            .iter_mut()
            .find(|field_card| field_card.card.id() == id)
        {
            field_card.state = state;
        }
    }

    pub fn set_card_battle_state(&mut self, id: ObjectId, state: Option<FieldBattleState>) {
        if let Some(field_card) = self
            .cards
            .iter_mut()
            .find(|field_card| field_card.card.id() == id)
        {
            field_card.battle = state;
        }
    }

    pub fn items(&self) -> impl Iterator<Item = &FieldItem<Card>> {
        self.cards.iter()
    }

    pub fn items_mut(&mut self) -> impl Iterator<Item = &mut FieldItem<Card>> {
        self.cards.iter_mut()
    }
}

impl CardZone for Field {
    type Item = Card;

    fn len(&self) -> usize {
        self.cards.len()
    }

    fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = &Card> {
        self.cards.iter().map(|field_card| &field_card.card)
    }

    fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Card> {
        self.cards.iter_mut().map(|field_card| &mut field_card.card)
    }

    fn push(&mut self, card: Card) {
        self.cards.push(FieldItem {
            card,
            state: FieldState::Active,
            battle: None,
        });
    }

    fn remove(&mut self, id: ObjectId) -> Option<Card> {
        let index = self
            .cards
            .iter()
            .position(|field_card| field_card.card.id() == id)?;
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
                .map(|field_card| FieldItem {
                    card: field_card.card.duplicate(),
                    state: field_card.state,
                    battle: field_card.battle,
                })
                .collect(),
        }
    }
}

impl CardSequence for Field {
    fn remove_top(&mut self) -> Option<Card> {
        self.cards.pop().map(|field_card| field_card.card)
    }

    fn add_top(&mut self, card: Card) {
        self.cards.push(FieldItem {
            card,
            state: FieldState::Active,
            battle: None,
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldItem<T: CardId> {
    pub card: T,
    pub state: FieldState,
    pub battle: Option<FieldBattleState>,
}

impl<T> CardId for FieldItem<T>
where
    T: CardId,
{
    fn id(&self) -> ObjectId {
        self.card.id()
    }
}

impl<T> Score for FieldItem<T>
where
    T: CardId + Score,
{
    fn score(&self) -> i32 {
        self.card.score()
            + match self.state {
                FieldState::Active => 1,
                FieldState::Exhausted => 0,
            }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum FieldState {
    Active,
    Exhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum FieldBattleState {
    Attacking,
    Blocking { attacker: ObjectId },
    Attacked,
}
