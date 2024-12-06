use crate::{
    ability::{AnonymousAbility, KeywordAbility},
    color::Color,
    effect::{Effect, NoEffect},
};
use bincode::{
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{DecodeError, EncodeError},
    BorrowDecode, Decode, Encode,
};
use core::fmt;
use serde::{Deserialize, Serialize};
use strum::Display;
use tinystr::TinyAsciiStr;

pub mod effect;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArchetypeId(TinyAsciiStr<8>);

impl fmt::Display for ArchetypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Encode for ArchetypeId {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&u64::from_be_bytes(*self.0.all_bytes()), encoder)?;
        Ok(())
    }
}

impl Decode for ArchetypeId {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let bytes = <u64 as Decode>::decode(decoder)?.to_be_bytes();
        let len = bytes.iter().position(|&b| b == 0).unwrap_or(8);
        Ok(Self(TinyAsciiStr::from_bytes_lossy(&bytes[..len])))
    }
}

impl<'de> BorrowDecode<'de> for ArchetypeId {
    fn borrow_decode<D: BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let bytes = <u64 as Decode>::decode(decoder)?.to_be_bytes();
        let len = bytes.iter().position(|&b| b == 0).unwrap_or(8);
        Ok(Self(TinyAsciiStr::from_bytes_lossy(&bytes[..len])))
    }
}

impl ArchetypeId {
    pub fn new(id: &str) -> Self {
        Self(TinyAsciiStr::from_bytes_lossy(id.as_bytes()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl AsRef<str> for ArchetypeId {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardArchetype {
    pub id: ArchetypeId,
    pub name: String,
    pub safe_name: String,
    pub attribute: CardAttribute,
    pub effect: fn() -> Box<dyn Effect>,
}

impl PartialOrd for CardArchetype {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CardArchetype {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.attribute
            .cmp(&other.attribute)
            .then(self.id.cmp(&other.id))
    }
}

impl Default for CardArchetype {
    fn default() -> Self {
        Self {
            id: ArchetypeId::new(""),
            name: String::new(),
            safe_name: String::new(),
            attribute: CardAttribute::default(),
            effect: no_effect(),
        }
    }
}

fn no_effect() -> fn() -> Box<dyn Effect> {
    NoEffect::NEW
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardAttribute {
    pub color: Color,
    pub cost: u8,
    pub card_type: CardType,
    pub creature_type: Option<CreatureType>,
    pub abilities: Vec<KeywordAbility>,
    pub anon_abilities: Vec<AnonymousAbility>,
    pub power: Option<u8>,
    pub is_token: bool,
    pub styles: Vec<CardStyle>,
}

impl Default for CardAttribute {
    fn default() -> Self {
        Self {
            color: Color::COLORLESS,
            cost: 1,
            card_type: CardType::Hex,
            creature_type: None,
            abilities: Vec::new(),
            anon_abilities: Vec::new(),
            power: None,
            is_token: false,
            styles: Vec::new(),
        }
    }
}

impl PartialOrd for CardAttribute {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CardAttribute {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.cost
            .cmp(&other.cost)
            .then(self.color.cmp(&other.color))
            .then(self.card_type.cmp(&other.card_type))
            .then(self.creature_type.cmp(&other.creature_type))
            .then(self.power.cmp(&other.power))
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
#[serde(rename_all = "snake_case")]
pub enum CardType {
    Creature,
    Hex,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Hash,
    Encode,
    Decode,
)]
#[serde(rename_all = "snake_case")]
pub enum CreatureType {
    Mutant,
    Cyborg,
    Robot,
    Ghost,
    Program,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardStyle {
    pub artwork: u8,
    pub artist: Option<String>,
}
