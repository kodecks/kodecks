use crate::{
    ability::{AbilityList, AnonymousAbility, KeywordAbility},
    card::{Card, CardArchetype, CardType, CreatureType},
    color::Color,
    linear::Linear,
    zone::CardZone,
};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputedAttribute {
    pub color: Color,
    pub cost: Linear<u8>,
    pub card_type: CardType,
    pub creature_type: Option<CreatureType>,
    pub abilities: AbilityList<KeywordAbility>,
    pub anon_abilities: AbilityList<AnonymousAbility>,
    pub power: Option<Linear<u32>>,
    pub shields: Option<Linear<u8>>,
}

impl From<&CardArchetype> for ComputedAttribute {
    fn from(archetype: &CardArchetype) -> Self {
        Self {
            color: archetype.attribute.color,
            cost: archetype.attribute.cost.into(),
            card_type: archetype.attribute.card_type,
            creature_type: archetype.attribute.creature_type,
            abilities: archetype.attribute.abilities.iter().copied().collect(),
            anon_abilities: archetype.attribute.anon_abilities.iter().copied().collect(),
            power: archetype.attribute.power.map(Linear::from),
            shields: archetype.attribute.shields.map(Linear::from),
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

    pub fn current_power(&self) -> u32 {
        self.power.map(|power| power.value()).unwrap_or(0)
    }

    pub fn current_shields(&self) -> u8 {
        self.shields.map(|shields| shields.value()).unwrap_or(0)
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ComputedFlags: u32 {
        const TARGETABLE = 0b00000001;
    }
}

impl ComputedFlags {
    pub fn is_targetable(&self) -> bool {
        self.contains(Self::TARGETABLE)
    }
}

pub trait ComputedSequence: CardZone<Item = Card> {
    fn update_computed(&mut self, states: Vec<ComputedAttribute>) {
        for (card, state) in self.iter_mut().zip(states) {
            card.set_computed(state);
        }
    }
}

impl<T> ComputedSequence for T where T: CardZone<Item = Card> {}
