use crate::{
    ability::{AbilityList, AnonymousAbility, KeywordAbility},
    card::{Card, CardArchetype, CardType},
    color::Color,
    linear::Linear,
    zone::CardZone,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputedAttribute {
    pub color: Color,
    pub cost: Linear<u8>,
    pub card_type: CardType,
    pub abilities: AbilityList<KeywordAbility>,
    pub anon_abilities: AbilityList<AnonymousAbility>,
    pub power: Option<Linear<u32>>,
}

impl From<&CardArchetype> for ComputedAttribute {
    fn from(archetype: &CardArchetype) -> Self {
        Self {
            color: archetype.attribute.color,
            cost: archetype.attribute.cost.into(),
            card_type: archetype.attribute.card_type,
            abilities: archetype.attribute.abilities.iter().copied().collect(),
            anon_abilities: archetype.attribute.anon_abilities.iter().copied().collect(),
            power: archetype.attribute.power.map(Linear::from),
        }
    }
}

impl ComputedAttribute {
    pub fn is_creature(&self) -> bool {
        matches!(self.card_type, CardType::Creature)
    }

    pub fn is_hex(&self) -> bool {
        matches!(self.card_type, CardType::Hex)
    }
}

pub trait ComputedSequence: CardZone<Item = Card> {
    fn update_computed(&mut self, states: Vec<ComputedAttribute>) {
        for (card, state) in self.iter_mut().zip(states) {
            *card.computed_mut() = state;
        }
    }
}

impl<T> ComputedSequence for T where T: CardZone<Item = Card> {}
