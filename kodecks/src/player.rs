use crate::{
    ability::{AbilityList, PlayerAbility},
    card::{Card, CardSnapshot},
    config::DebugFlags,
    deck::{Deck, DeckList},
    env::GameState,
    field::{Field, FieldItem, FieldState},
    graveyard::Graveyard,
    hand::{Hand, HandItem},
    id::ObjectId,
    shard::ShardList,
    zone::{CardZone, Zone},
};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt};
use tinystr::TinyAsciiStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlayerId(TinyAsciiStr<16>);

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PlayerId {
    pub fn new(id: &str) -> Self {
        Self(TinyAsciiStr::from_bytes_lossy(id.as_bytes()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub id: PlayerId,
    pub deck: DeckList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerList<T> {
    player_in_turn: PlayerId,
    players: Vec<T>,
}

impl<T> PlayerList<T> {
    pub fn new<I>(player_in_turn: PlayerId, iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            player_in_turn,
            players: iter.into_iter().collect(),
        }
    }

    pub fn player_in_turn(&self) -> PlayerId {
        self.player_in_turn
    }

    pub fn set_player_in_turn(&mut self, player: PlayerId) {
        self.player_in_turn = player;
    }
}

pub struct PlayerListIter<'a, T> {
    players: VecDeque<&'a T>,
}

impl<'a, T> PlayerListIter<'a, T>
where
    T: PlayerItem,
{
    pub fn new(player_in_turn: PlayerId, players: &'a [T]) -> Self {
        let pos = players
            .iter()
            .position(|player| player.id() == player_in_turn)
            .unwrap_or(0);
        let mut players = players.iter().collect::<VecDeque<_>>();
        players.rotate_left(pos);
        Self { players }
    }
}

impl<'a, T> Iterator for PlayerListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.players.pop_front()
    }
}

pub struct PlayerListMutIter<'a, T> {
    players: VecDeque<&'a mut T>,
}

impl<'a, T> PlayerListMutIter<'a, T>
where
    T: PlayerItem,
{
    pub fn new(player_in_turn: PlayerId, players: &'a mut [T]) -> Self {
        let pos = players
            .iter()
            .position(|player| player.id() == player_in_turn)
            .unwrap_or(0);
        let mut players = players.iter_mut().collect::<VecDeque<_>>();
        players.rotate_left(pos);
        Self { players }
    }
}

impl<'a, T> Iterator for PlayerListMutIter<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.players.pop_front()
    }
}

impl<T> PlayerList<T>
where
    T: PlayerItem,
{
    pub fn get(&self, player: PlayerId) -> &T {
        self.players
            .iter()
            .find(|item| item.id() == player)
            .unwrap()
    }

    pub fn get_mut(&mut self, player: PlayerId) -> &mut T {
        let index = self
            .players
            .iter()
            .position(|item| item.id() == player)
            .unwrap();
        &mut self.players[index]
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        PlayerListIter::new(self.player_in_turn, &self.players)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        PlayerListMutIter::new(self.player_in_turn, &mut self.players)
    }

    pub fn next(&self, id: PlayerId) -> PlayerId {
        let pos = self
            .players
            .iter()
            .position(|player| player.id() == id)
            .unwrap_or(0);
        let next = (pos + 1) % self.players.len();
        self.players[next].id()
    }

    pub fn push(&mut self, player: T) {
        self.players.push(player);
    }
}

pub trait PlayerItem {
    fn id(&self) -> PlayerId;
}

#[derive(Debug)]
pub struct PlayerState {
    pub id: PlayerId,
    pub deck: Deck,
    pub hand: Hand,
    pub graveyard: Graveyard,
    pub field: Field,
    pub shards: ShardList,
    pub stats: PlayerStats,
    pub counters: PlayerCounters,
    pub condition: Option<PlayerCondition>,
    pub abilities: AbilityList<PlayerAbility>,
}

impl PlayerItem for PlayerState {
    fn id(&self) -> PlayerId {
        self.id
    }
}

impl Clone for PlayerState {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            deck: self.deck.duplicate(),
            hand: self.hand.duplicate(),
            graveyard: self.graveyard.duplicate(),
            field: self.field.duplicate(),
            shards: self.shards.clone(),
            stats: self.stats,
            counters: self.counters,
            condition: self.condition,
            abilities: self.abilities.clone(),
        }
    }
}

impl PlayerState {
    pub fn new(id: PlayerId) -> Self {
        PlayerState {
            id,
            deck: Deck::default(),
            hand: Hand::default(),
            graveyard: Graveyard::default(),
            field: Field::default(),
            shards: ShardList::new(),
            stats: PlayerStats::default(),
            counters: PlayerCounters::default(),
            condition: None,
            abilities: AbilityList::default(),
        }
    }

    pub fn find_card(&self, card: ObjectId) -> Option<&Card> {
        self.deck
            .get(card)
            .or_else(|| self.hand.get(card))
            .or_else(|| self.graveyard.get(card))
            .or_else(|| self.field.get(card))
    }

    pub fn find_card_mut(&mut self, card: ObjectId) -> Option<&mut Card> {
        self.deck
            .get_mut(card)
            .or_else(|| self.hand.get_mut(card))
            .or_else(|| self.graveyard.get_mut(card))
            .or_else(|| self.field.get_mut(card))
    }

    pub fn find_zone(&self, card: ObjectId) -> Option<Zone> {
        if self.deck.contains(card) {
            Some(Zone::Deck)
        } else if self.hand.contains(card) {
            Some(Zone::Hand)
        } else if self.field.contains(card) {
            Some(Zone::Field)
        } else if self.graveyard.contains(card) {
            Some(Zone::Graveyard)
        } else {
            None
        }
    }

    pub fn reset_counters(&mut self) {
        self.counters = PlayerCounters::default();
    }

    pub fn castable_cards<'a>(
        &'a self,
        state: &'a GameState,
    ) -> impl Iterator<Item = ObjectId> + 'a {
        self.hand
            .items()
            .filter(|item| {
                state.config.debug.contains(DebugFlags::IGNORE_COST)
                    || item.card.computed().cost.value() == 0
                    || self.shards.get(item.card.computed().color)
                        >= item.card.computed().cost.value() as u32
            })
            .filter(|item| !item.card.computed().is_creature() || self.counters.cast_creatures == 0)
            .filter(|item| item.card.effect().is_castable(state, &item.card))
            .map(|item| item.card.id())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerStats {
    pub life: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self { life: 2000 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerCondition {
    Win,
    Lose,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PlayerCounters {
    pub draw: u32,
    pub cast_creatures: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerZone {
    pub player: PlayerId,
    pub zone: Zone,
}

impl PlayerZone {
    pub fn new(player: PlayerId, zone: Zone) -> Self {
        Self { player, zone }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalPlayerState {
    pub id: PlayerId,
    pub deck: usize,
    pub hand: Vec<HandItem<CardSnapshot>>,
    pub graveyard: Vec<CardSnapshot>,
    pub field: Vec<FieldItem<CardSnapshot>>,
    pub shards: ShardList,
    pub stats: PlayerStats,
}

impl PlayerItem for LocalPlayerState {
    fn id(&self) -> PlayerId {
        self.id
    }
}

impl LocalPlayerState {
    pub fn new(state: &PlayerState, private: bool) -> Self {
        Self {
            id: state.id,
            deck: state.deck.len(),
            hand: state
                .hand
                .items()
                .map(|item| {
                    let card = item.card.snapshot();
                    let card = if private { card } else { card.redacted() };
                    (card, item.cost_delta)
                })
                .map(|(card, cost_delta)| HandItem { card, cost_delta })
                .collect(),
            graveyard: state.graveyard.iter().map(|card| card.snapshot()).collect(),
            field: state
                .field
                .items()
                .map(|item| FieldItem {
                    card: item.card.snapshot(),
                    state: item.state,
                    battle: item.battle,
                })
                .collect(),
            shards: state.shards.clone(),
            stats: state.stats,
        }
    }

    pub fn find_card(&self, card: ObjectId) -> Option<&CardSnapshot> {
        self.field
            .iter()
            .find(|item| item.id == card)
            .or_else(|| self.graveyard.iter().find(|item| item.id == card))
            .or_else(|| self.hand.iter().find(|item| item.id == card))
    }

    pub fn cards(&self) -> impl Iterator<Item = &CardSnapshot> {
        self.hand
            .iter()
            .chain(self.graveyard.iter())
            .chain(self.field.iter())
    }

    pub fn find_zone(&self, card: ObjectId) -> Option<Zone> {
        if self.hand.iter().any(|item| item.id == card) {
            return Some(Zone::Hand);
        }
        if self.field.iter().any(|item| item.id == card) {
            return Some(Zone::Field);
        }
        if self.graveyard.iter().any(|item| item.id == card) {
            return Some(Zone::Graveyard);
        }
        None
    }
}

impl CardZone for Vec<CardSnapshot> {
    type Item = CardSnapshot;

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn push(&mut self, card: CardSnapshot) {
        self.push(card);
    }

    fn remove(&mut self, id: ObjectId) -> Option<Self::Item> {
        let index = self.iter().position(|card| card.id == id)?;
        Some(self.remove(index))
    }

    fn duplicate(&self) -> Self
    where
        Self: Sized,
    {
        self.clone()
    }

    fn get(&self, id: ObjectId) -> Option<&Self::Item> {
        self.iter().find(|card| card.id == id)
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = &Self::Item> {
        (self as &[CardSnapshot]).iter()
    }

    fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Self::Item> {
        (self as &mut [CardSnapshot]).iter_mut()
    }
}

impl CardZone for Vec<FieldItem<CardSnapshot>> {
    type Item = CardSnapshot;

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn push(&mut self, card: CardSnapshot) {
        self.push(FieldItem {
            card,
            state: FieldState::Active,
            battle: None,
        });
    }

    fn remove(&mut self, id: ObjectId) -> Option<Self::Item> {
        let index = self.iter().position(|card| card.id == id)?;
        Some(self.remove(index).card)
    }

    fn duplicate(&self) -> Self
    where
        Self: Sized,
    {
        self.clone()
    }

    fn get(&self, id: ObjectId) -> Option<&Self::Item> {
        self.iter().find(|card| card.id == id)
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = &Self::Item> {
        (self as &[FieldItem<CardSnapshot>])
            .iter()
            .map(|item| &item.card)
    }

    fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Self::Item> {
        (self as &mut [FieldItem<CardSnapshot>])
            .iter_mut()
            .map(|item| &mut item.card)
    }
}

impl CardZone for Vec<HandItem<CardSnapshot>> {
    type Item = CardSnapshot;

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn push(&mut self, card: CardSnapshot) {
        self.push(HandItem {
            card,
            cost_delta: 0,
        });
    }

    fn remove(&mut self, id: ObjectId) -> Option<Self::Item> {
        let index = self.iter().position(|item| item.id == id)?;
        Some(self.remove(index).card)
    }

    fn duplicate(&self) -> Self
    where
        Self: Sized,
    {
        self.clone()
    }

    fn get(&self, id: ObjectId) -> Option<&Self::Item> {
        self.iter().find(|item| item.id == id)
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = &Self::Item> {
        (self as &[HandItem<CardSnapshot>])
            .iter()
            .map(|item| &item.card)
    }

    fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Self::Item> {
        (self as &mut [HandItem<CardSnapshot>])
            .iter_mut()
            .map(|item| &mut item.card)
    }
}
