use crate::{
    ability::{AnonymousAbility, KeywordAbility},
    color::Color,
    computed::{ComputedAttribute, ComputedFlags},
    deck::DeckItem,
    effect::{Effect, NoEffect},
    event::EventFilter,
    id::{CardId, ObjectId, ObjectIdCounter, TimedObjectId},
    linear::Linear,
    player::{PlayerMask, PlayerZone},
    score::Score,
    zone::Zone,
};
use bincode::{
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{DecodeError, EncodeError},
    BorrowDecode, Decode, Encode,
};
use core::fmt;
use num::Zero;
use serde::{Deserialize, Serialize};
use std::{ops::Index, sync::LazyLock};
use strum::Display;
use tinystr::TinyAsciiStr;

pub type CardMap = phf::Map<&'static str, fn() -> &'static CardArchetype>;

pub struct Catalog {
    pub str_index: &'static CardMap,
}

impl Catalog {
    pub fn iter(&self) -> impl Iterator<Item = &'static CardArchetype> {
        self.str_index.values().map(|entry| entry())
    }

    pub fn get(&self, safe_name: &str) -> Option<&'static CardArchetype> {
        self.str_index.get(safe_name).map(|entry| entry())
    }

    pub fn contains(&self, safe_name: &str) -> bool {
        self.str_index.contains_key(safe_name)
    }
}

impl Index<&str> for Catalog {
    type Output = CardArchetype;

    fn index(&self, safe_name: &str) -> &Self::Output {
        self.get(safe_name).unwrap_or(CardArchetype::NONE())
    }
}

impl Index<ArchetypeId> for Catalog {
    type Output = CardArchetype;

    fn index(&self, short_id: ArchetypeId) -> &Self::Output {
        self.index(short_id.as_str())
    }
}

pub struct Card {
    id: ObjectId,
    owner: u8,
    zone: PlayerZone,
    controller: u8,
    archetype: &'static CardArchetype,
    style: u8,
    computed: ComputedAttribute,
    flags: ComputedFlags,
    event_filter: EventFilter,
    effect: Box<dyn Effect>,
    revealed: PlayerMask,
    timestamp: u64,
    is_token: bool,
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{} {}>", self.id, self.archetype.name)
    }
}

impl Card {
    pub fn new(
        counter: &mut ObjectIdCounter,
        item: &DeckItem,
        archetype: &'static CardArchetype,
        style: u8,
        owner: u8,
    ) -> Self {
        let effect = (archetype.effect)();
        Self {
            id: counter.allocate(item.base_id),
            owner,
            zone: PlayerZone::new(owner, Zone::Deck),
            controller: owner,
            archetype,
            style,
            computed: archetype.into(),
            flags: ComputedFlags::empty(),
            event_filter: effect.event_filter(),
            effect,
            revealed: PlayerMask::default(),
            timestamp: 0,
            is_token: false,
        }
    }

    pub fn new_token(id: ObjectId, archetype: &'static CardArchetype, owner: u8) -> Self {
        let effect = (archetype.effect)();
        Self {
            id,
            owner,
            zone: PlayerZone::new(owner, Zone::Deck),
            controller: owner,
            archetype,
            style: 0,
            computed: archetype.into(),
            flags: ComputedFlags::empty(),
            event_filter: effect.event_filter(),
            revealed: PlayerMask::default(),
            effect,
            timestamp: 0,
            is_token: true,
        }
    }

    pub fn id(&self) -> ObjectId {
        self.id
    }

    pub fn owner(&self) -> u8 {
        self.owner
    }

    pub fn controller(&self) -> u8 {
        self.controller
    }

    pub fn zone(&self) -> &PlayerZone {
        &self.zone
    }

    pub fn set_zone(&mut self, zone: PlayerZone) {
        self.zone = zone;
        match zone.zone {
            Zone::Hand => {
                self.revealed.set(zone.player, true);
            }
            Zone::Field | Zone::Graveyard => {
                self.revealed.set_all(true);
            }
            _ => (),
        }
    }

    pub fn archetype(&self) -> &'static CardArchetype {
        self.archetype
    }

    pub fn computed(&self) -> &ComputedAttribute {
        &self.computed
    }

    pub fn set_computed(&mut self, computed: ComputedAttribute) {
        self.computed = computed;

        let mut flags = ComputedFlags::empty();
        let stealth = self.zone.zone == Zone::Field
            && self.computed.abilities.contains(&KeywordAbility::Stealth);
        flags.set(ComputedFlags::TARGETABLE, !stealth);
        self.flags = flags;
    }

    pub fn reset_computed(&mut self) {
        self.set_computed(self.archetype.into());
    }

    pub fn flags(&self) -> ComputedFlags {
        self.flags
    }

    pub fn event_filter(&self) -> EventFilter {
        self.event_filter
    }

    pub fn effect(&self) -> Box<dyn Effect> {
        #[allow(clippy::deref_addrof)]
        dyn_clone::clone_box(&**&self.effect)
    }

    pub fn revealed(&self) -> PlayerMask {
        self.revealed
    }

    pub fn set_effect(&mut self, effect: Box<dyn Effect>) {
        self.effect = effect;
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
    }

    pub fn is_token(&self) -> bool {
        self.is_token
    }

    pub fn snapshot(&self) -> CardSnapshot {
        CardSnapshot {
            id: self.id,
            archetype_id: self.archetype.id,
            style: self.style,
            controller: self.controller,
            owner: self.owner,
            revealed: self.revealed,
            computed: Some(self.computed.clone()),
            timestamp: self.timestamp,
            is_token: self.is_token,
        }
    }

    pub fn renew_id(&mut self, counter: &mut ObjectIdCounter) {
        self.id = counter.allocate(Some(self.id));
        self.revealed.set_all(false);
    }
}

impl Clone for Card {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            owner: self.owner,
            zone: self.zone,
            controller: self.controller,
            archetype: self.archetype,
            style: self.style,
            computed: self.computed.clone(),
            flags: self.flags,
            event_filter: self.event_filter,
            effect: self.effect(),
            revealed: self.revealed,
            timestamp: self.timestamp,
            is_token: self.is_token,
        }
    }
}

impl CardId for Card {
    fn id(&self) -> ObjectId {
        self.id
    }

    fn timed_id(&self) -> TimedObjectId {
        TimedObjectId {
            id: self.id,
            timestamp: self.timestamp,
        }
    }
}

impl AsRef<Card> for Card {
    fn as_ref(&self) -> &Card {
        self
    }
}

impl AsMut<Card> for Card {
    fn as_mut(&mut self) -> &mut Card {
        self
    }
}

impl Score for Card {
    fn score(&self) -> i32 {
        self.computed.abilities.score()
            + self.computed.anon_abilities.score()
            + self.computed.power.map(|power| power.value()).unwrap_or(0) as i32 / 100
            + self
                .computed
                .shields
                .map(|shields| shields.value())
                .unwrap_or(0) as i32
                * 2
            + if self.computed.is_creature() { 1 } else { 0 }
    }
}

pub fn safe_name(name: &str) -> Result<String, idna::Errors> {
    idna::domain_to_ascii(&name.replace(' ', "-"))
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct CardSnapshot {
    pub id: ObjectId,
    pub archetype_id: ArchetypeId,
    pub style: u8,
    pub controller: u8,
    pub owner: u8,
    pub revealed: PlayerMask,
    pub computed: Option<ComputedAttribute>,
    pub timestamp: u64,
    pub is_token: bool,
}

impl CardId for CardSnapshot {
    fn id(&self) -> ObjectId {
        self.id
    }

    fn timed_id(&self) -> TimedObjectId {
        TimedObjectId {
            id: self.id,
            timestamp: self.timestamp,
        }
    }
}

impl CardSnapshot {
    pub fn redacted(self, viewer: u8) -> Self {
        if self.revealed.contains(viewer) {
            self
        } else {
            Self {
                archetype_id: ArchetypeId::new(""),
                computed: None,
                timestamp: 0,
                ..self
            }
        }
    }

    pub fn color(&self) -> Color {
        self.computed
            .as_ref()
            .map(|c| c.color)
            .unwrap_or(Color::empty())
    }

    pub fn cost(&self) -> Linear<u8> {
        self.computed.as_ref().map(|c| c.cost).unwrap_or_default()
    }

    pub fn power(&self) -> Option<Linear<u32>> {
        self.computed.as_ref().and_then(|c| c.power)
    }

    pub fn shields(&self) -> Option<Linear<u8>> {
        self.computed.as_ref().and_then(|c| c.shields)
    }
}

impl fmt::Display for CardSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let computed = if let Some(computed) = &self.computed {
            computed
        } else {
            return write!(f, "<???>",);
        };
        let color = match computed.color {
            Color::RED => "R",
            Color::YELLOW => "Y",
            Color::GREEN => "G",
            Color::BLUE => "B",
            _ => "--",
        };
        let clock = format!(" {}", self.power().map(|p| p.value()).unwrap_or(0));
        write!(f, "<({color}{}){clock} {}>", computed.cost.value(), self.id)
    }
}

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
        Encode::encode(self.0.as_str(), encoder)?;
        Ok(())
    }
}

impl Decode for ArchetypeId {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self(TinyAsciiStr::from_bytes_lossy(
            <String as Decode>::decode(decoder)?.as_bytes(),
        )))
    }
}

impl<'de> BorrowDecode<'de> for ArchetypeId {
    fn borrow_decode<D: BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self(TinyAsciiStr::from_bytes_lossy(
            <String as Decode>::decode(decoder)?.as_bytes(),
        )))
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardArchetype {
    pub id: ArchetypeId,
    pub name: &'static str,
    pub safe_name: &'static str,
    pub attribute: CardAttribute,
    pub effect: fn() -> Box<dyn Effect>,
}

impl CardArchetype {
    pub const NONE: fn() -> &'static CardArchetype = || {
        static CACHE: LazyLock<CardArchetype> = LazyLock::new(CardArchetype::default);
        &CACHE
    };
}

impl Default for CardArchetype {
    fn default() -> Self {
        Self {
            id: ArchetypeId::new(""),
            name: "",
            safe_name: "",
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
    pub abilities: &'static [KeywordAbility],
    pub anon_abilities: &'static [AnonymousAbility],
    pub power: Option<u32>,
    pub shields: Option<u8>,
    pub is_token: bool,
    pub alt_styles: &'static [CardStyle],
}

impl Default for CardAttribute {
    fn default() -> Self {
        Self {
            color: Color::COLORLESS,
            cost: 0,
            card_type: CardType::Hex,
            creature_type: None,
            abilities: &[],
            anon_abilities: &[],
            power: None,
            shields: None,
            is_token: false,
            alt_styles: &[],
        }
    }
}

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum CardType {
    Creature,
    Hex,
}

#[derive(
    Debug, Clone, Copy, Display, PartialEq, Eq, Serialize, Deserialize, Hash, Encode, Decode,
)]
#[serde(rename_all = "snake_case")]
pub enum CreatureType {
    Mutant,
    Cyborg,
    Robot,
    Ghost,
    Program,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardStyle {
    pub illustration: u8,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Encode, Decode, Hash,
)]
pub struct CardEntry {
    pub archetype_id: ArchetypeId,
    #[serde(default, skip_serializing_if = "Zero::is_zero")]
    pub style: u8,
}
