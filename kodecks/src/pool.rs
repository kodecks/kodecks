use crate::{card::ArchetypeId, deck::DeckList};
use bincode::{
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{DecodeError, EncodeError},
    BorrowDecode, Decode, Encode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, Hash)]
pub struct CardPool(Vec<(CardPoolEntry, PoolStatus)>);

impl CardPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn verify(&self, deck: &DeckList) -> bool {
        let mut count = HashMap::new();
        for item in &deck.cards {
            let entry = count.entry(item.card.archetype_id).or_insert(0);
            *entry += 1;
        }
        for (entry, status) in &self.0 {
            if let CardPoolEntry::Card(id) = entry {
                let status: u8 = (*status).into();
                if let Some(&value) = count.get(id) {
                    if value > status {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl Default for CardPool {
    fn default() -> Self {
        Self(vec![(CardPoolEntry::CoreSet, PoolStatus::Legal)])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, Hash)]
pub enum CardPoolEntry {
    CoreSet,
    Card(ArchetypeId),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolStatus {
    #[default]
    Legal,
    Limited(u8),
    Illegal,
}

impl From<u8> for PoolStatus {
    fn from(value: u8) -> Self {
        match value {
            255 => PoolStatus::Legal,
            1..=254 => PoolStatus::Limited(value),
            0 => PoolStatus::Illegal,
        }
    }
}

impl From<PoolStatus> for u8 {
    fn from(value: PoolStatus) -> Self {
        match value {
            PoolStatus::Legal => 255,
            PoolStatus::Limited(value) => value,
            PoolStatus::Illegal => 0,
        }
    }
}

impl Serialize for PoolStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        u8::from(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PoolStatus {
    fn deserialize<D>(deserializer: D) -> Result<PoolStatus, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        u8::deserialize(deserializer).map(PoolStatus::from)
    }
}

impl Encode for PoolStatus {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&u8::from(*self), encoder)?;
        Ok(())
    }
}

impl Decode for PoolStatus {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let value: u8 = Decode::decode(decoder)?;
        Ok(PoolStatus::from(value))
    }
}

impl<'de> BorrowDecode<'de> for PoolStatus {
    fn borrow_decode<D: BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let value: u8 = Decode::decode(decoder)?;
        Ok(PoolStatus::from(value))
    }
}
