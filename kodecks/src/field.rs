use crate::{
    card::Card,
    id::{CardId, ObjectId, TimedObjectId},
    list::CardList,
    score::Score,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum::Display;

impl CardList<FieldItem<Card>> {
    pub fn active_cards(&self) -> impl Iterator<Item = &Card> {
        self.items().filter_map(|field_card| {
            if field_card.state == FieldState::Active {
                Some(&field_card.card)
            } else {
                None
            }
        })
    }

    pub fn attacking_cards(&self) -> impl Iterator<Item = &Card> {
        self.items().filter_map(|field_card| {
            if let Some(FieldBattleState::Attacking) = field_card.battle {
                Some(&field_card.card)
            } else {
                None
            }
        })
    }

    pub fn find_blocker(&self, attacker: ObjectId) -> Option<&Card> {
        self.items().find_map(|field_card| match field_card.battle {
            Some(FieldBattleState::Blocking { attacker: id }) if id == attacker => {
                Some(&field_card.card)
            }
            _ => None,
        })
    }

    pub fn set_card_state(&mut self, id: ObjectId, state: FieldState) {
        if let Some(field_card) = self
            .items_mut()
            .find(|field_card| field_card.card.id() == id)
        {
            field_card.state = state;
        }
    }

    pub fn set_card_battle_state(&mut self, id: ObjectId, state: Option<FieldBattleState>) {
        if let Some(field_card) = self
            .items_mut()
            .find(|field_card| field_card.card.id() == id)
        {
            field_card.battle = state;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct FieldItem<T: CardId> {
    pub card: T,
    pub state: FieldState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battle: Option<FieldBattleState>,
}

impl<T> CardId for FieldItem<T>
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

impl AsRef<Card> for FieldItem<Card> {
    fn as_ref(&self) -> &Card {
        &self.card
    }
}

impl AsMut<Card> for FieldItem<Card> {
    fn as_mut(&mut self) -> &mut Card {
        &mut self.card
    }
}

impl From<Card> for FieldItem<Card> {
    fn from(card: Card) -> Self {
        Self {
            card,
            state: FieldState::Active,
            battle: None,
        }
    }
}

impl From<FieldItem<Card>> for Card {
    fn from(item: FieldItem<Card>) -> Self {
        item.card
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

#[derive(
    Debug, Clone, Copy, Default, Display, PartialEq, Eq, Serialize, Deserialize, Encode, Decode,
)]
#[serde(rename_all = "snake_case")]
pub enum FieldState {
    #[default]
    Active,
    Exhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum FieldBattleState {
    Attacking,
    Blocking { attacker: ObjectId },
    Attacked,
}
