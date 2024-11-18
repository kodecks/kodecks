use crate::{
    ability::KeywordAbility,
    archetype::{ArchetypeId, CardArchetype},
    color::Color,
    computed::{ComputedAttribute, ComputedFlags},
    deck::DeckItem,
    effect::Effect,
    event::EventFilter,
    id::{CardId, ObjectId, ObjectIdCounter, TimedCardId, TimedObjectId},
    linear::Linear,
    player::{PlayerMask, Zone},
    score::Score,
    zone::ZoneKind,
};
use bincode::{Decode, Encode};
use core::fmt;
use num::Zero;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct Card {
    id: ObjectId,
    owner: u8,
    zone: Zone,
    archetype: Arc<CardArchetype>,
    style: u8,
    computed: ComputedAttribute,
    flags: ComputedFlags,
    event_filter: EventFilter,
    effect: Box<dyn Effect>,
    revealed: PlayerMask,
    timestamp: u16,
    hand_cost_delta: i8,
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
        archetype: Arc<CardArchetype>,
        style: u8,
        owner: u8,
    ) -> Self {
        let effect = (archetype.effect)();
        let computed = (&*archetype).into();
        Self {
            id: counter.allocate(item.base_id),
            owner,
            zone: Zone::new(owner, ZoneKind::Deck),
            archetype,
            style,
            computed,
            flags: ComputedFlags::empty(),
            event_filter: effect.event_filter(),
            effect,
            revealed: PlayerMask::default(),
            timestamp: 0,
            hand_cost_delta: 0,
            is_token: false,
        }
    }

    pub fn new_token(id: ObjectId, archetype: Arc<CardArchetype>, owner: u8) -> Self {
        let effect = (archetype.effect)();
        let computed = (&*archetype).into();
        Self {
            id,
            owner,
            zone: Zone::new(owner, ZoneKind::Deck),
            archetype,
            style: 0,
            computed,
            flags: ComputedFlags::empty(),
            event_filter: effect.event_filter(),
            revealed: PlayerMask::default(),
            effect,
            timestamp: 0,
            hand_cost_delta: 0,
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
        self.zone.player
    }

    pub fn zone(&self) -> &Zone {
        &self.zone
    }

    pub fn set_zone(&mut self, zone: Zone) {
        if self.zone == zone {
            return;
        }
        match zone.kind {
            ZoneKind::Hand => {
                self.revealed.set(zone.player, true);
                self.set_hand_cost_delta(0);
            }
            ZoneKind::Field | ZoneKind::Graveyard => {
                self.revealed.set_all(true);
            }
            _ => (),
        }
        if self.zone.kind != ZoneKind::Field || zone.kind != ZoneKind::Field {
            self.increment_timestamp();
            self.reset_computed();
        }
        self.zone = zone;
    }

    pub fn archetype(&self) -> &Arc<CardArchetype> {
        &self.archetype
    }

    pub fn computed(&self) -> &ComputedAttribute {
        &self.computed
    }

    pub fn set_computed(&mut self, computed: ComputedAttribute) {
        self.computed = computed;

        let mut flags = ComputedFlags::empty();
        let stealth = self.zone.kind == ZoneKind::Field
            && self.computed.abilities.contains(&KeywordAbility::Stealth);
        flags.set(ComputedFlags::TARGETABLE, !stealth);
        self.flags = flags;
    }

    pub fn reset_computed(&mut self) {
        self.set_computed((&*self.archetype).into());
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

    pub fn timestamp(&self) -> u16 {
        self.timestamp
    }

    pub fn increment_timestamp(&mut self) {
        self.timestamp = self.timestamp.wrapping_add(1);
    }

    pub fn is_token(&self) -> bool {
        self.is_token
    }

    pub fn hand_cost_delta(&self) -> i8 {
        self.hand_cost_delta
    }

    pub fn set_hand_cost_delta(&mut self, delta: i8) {
        self.hand_cost_delta = delta;
    }

    pub fn snapshot(&self) -> CardSnapshot {
        CardSnapshot {
            id: self.id,
            archetype_id: self.archetype.id,
            style: self.style,
            zone: self.zone,
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
            archetype: self.archetype.clone(),
            style: self.style,
            computed: self.computed.clone(),
            flags: self.flags,
            event_filter: self.event_filter,
            effect: self.effect(),
            revealed: self.revealed,
            timestamp: self.timestamp,
            hand_cost_delta: self.hand_cost_delta,
            is_token: self.is_token,
        }
    }
}

impl CardId for Card {
    fn id(&self) -> ObjectId {
        self.id
    }
}

impl TimedCardId for Card {
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
    pub zone: Zone,
    pub owner: u8,
    pub revealed: PlayerMask,
    pub computed: Option<ComputedAttribute>,
    pub timestamp: u16,
    pub is_token: bool,
}

impl CardId for CardSnapshot {
    fn id(&self) -> ObjectId {
        self.id
    }
}

impl TimedCardId for CardSnapshot {
    fn timed_id(&self) -> TimedObjectId {
        TimedObjectId {
            id: self.id,
            timestamp: self.timestamp,
        }
    }
}

impl CardSnapshot {
    pub fn new(archetype: &CardArchetype) -> Self {
        Self {
            id: 1u32.try_into().unwrap(),
            archetype_id: archetype.id,
            style: 0,
            zone: Zone {
                player: 0,
                kind: ZoneKind::Deck,
            },
            owner: 0,
            revealed: PlayerMask::default(),
            computed: Some(archetype.into()),
            timestamp: 0,
            is_token: false,
        }
    }

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

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Encode, Decode, Hash,
)]
pub struct CardEntry {
    pub archetype_id: ArchetypeId,
    #[serde(default, skip_serializing_if = "Zero::is_zero")]
    pub style: u8,
}
