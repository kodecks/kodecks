use crate::{
    card::Card,
    id::{ObjectId, TimedObjectId},
    list::CardList,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum::Display;

impl CardList<Card> {
    pub fn active_cards(&self) -> impl Iterator<Item = &Card> {
        self.items()
            .filter(|card| card.field_state() == FieldState::Active)
    }

    pub fn attacking_cards(&self) -> impl Iterator<Item = &Card> {
        self.items()
            .filter(|card| Some(FieldBattleState::Attacking) == card.battle_state())
    }

    pub fn find_blocker(&self, attacker: ObjectId) -> Option<&Card> {
        self.items().find(|card| {
            if let Some(FieldBattleState::Blocking { attacker: id }) = card.battle_state() {
                id.id == attacker
            } else {
                false
            }
        })
    }

    pub fn set_card_field_state(&mut self, id: ObjectId, state: FieldState) {
        if let Some(card) = self.items_mut().find(|card| card.id() == id) {
            card.set_field_state(state);
        }
    }

    pub fn set_card_battle_state(&mut self, id: ObjectId, state: Option<FieldBattleState>) -> bool {
        if let Some(card) = self.items_mut().find(|card| card.id() == id) {
            if card.battle_state() != state {
                card.set_battle_state(state);
                return true;
            }
        }
        false
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
    Blocking { attacker: TimedObjectId },
    Attacked,
}
