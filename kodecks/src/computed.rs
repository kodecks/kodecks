use crate::{
    ability::{AbilityList, AnonymousAbility, KeywordAbility},
    archetype::{CardArchetype, CardType, CreatureType},
    card::Card,
    color::Color,
    linear::Linear,
    zone::CardZone,
};
use bincode::{
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{DecodeError, EncodeError},
    BorrowDecode, Decode, Encode,
};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct ComputedAttribute {
    pub color: Color,
    pub cost: Linear<u8>,
    pub card_type: CardType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creature_type: Option<CreatureType>,
    #[serde(default, skip_serializing_if = "AbilityList::is_empty")]
    pub abilities: AbilityList<KeywordAbility>,
    #[serde(default, skip_serializing_if = "AbilityList::is_empty")]
    pub anon_abilities: AbilityList<AnonymousAbility>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power: Option<Linear<u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, )]
    pub struct ComputedFlags: u8 {
        const TARGETABLE = 0b00000001;
    }
}

impl Encode for ComputedFlags {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.bits(), encoder)?;
        Ok(())
    }
}

impl Decode for ComputedFlags {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self::from_bits_truncate(Decode::decode(decoder)?))
    }
}

impl<'de> BorrowDecode<'de> for ComputedFlags {
    fn borrow_decode<D: BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self::from_bits_truncate(Decode::decode(decoder)?))
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
