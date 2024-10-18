use crate::{
    ability::{AbilityList, PlayerAbility},
    card::{Card, CardSnapshot},
    deck::DeckList,
    env::{EndgameReason, GameState},
    field::{FieldItem, FieldState},
    hand::HandItem,
    id::ObjectId,
    list::CardList,
    profile::DebugFlags,
    shard::ShardList,
    zone::{CardZone, Zone},
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PlayerConfig {
    pub deck: DeckList,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PlayerList<T: 'static> {
    player_in_turn: u8,
    players: Vec<T>,
}

impl<T> PlayerList<T> {
    pub fn new<I>(player_in_turn: u8, iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            player_in_turn,
            players: iter.into_iter().collect(),
        }
    }

    pub fn set_player_in_turn(&mut self, player: u8) {
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
    pub fn new(player_in_turn: u8, players: &'a [T]) -> Self {
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
    pub fn new(player_in_turn: u8, players: &'a mut [T]) -> Self {
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
    pub fn get(&self, player: u8) -> &T {
        self.players
            .iter()
            .find(|item| item.id() == player)
            .unwrap()
    }

    pub fn get_mut(&mut self, player: u8) -> &mut T {
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

    pub fn next_id(&self, id: u8) -> u8 {
        let pos = self
            .players
            .iter()
            .position(|player| player.id() == id)
            .unwrap_or(0);
        let next = (pos + 1) % self.players.len();
        self.players[next].id()
    }

    pub fn next_player(&self, id: u8) -> &T {
        let next = self.next_id(id);
        self.get(next)
    }

    pub fn push(&mut self, player: T) {
        self.players.push(player);
    }

    pub fn player_in_turn(&self) -> &T {
        self.get(self.player_in_turn)
    }
}

pub trait PlayerItem {
    fn id(&self) -> u8;
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: u8,
    pub deck: CardList<Card>,
    pub hand: CardList<HandItem<Card>>,
    pub graveyard: CardList<Card>,
    pub field: CardList<FieldItem<Card>>,
    pub shards: ShardList,
    pub stats: PlayerStats,
    pub counters: PlayerCounters,
    pub endgame: Option<PlayerEndgameState>,
    pub abilities: AbilityList<PlayerAbility>,
}

impl PlayerItem for PlayerState {
    fn id(&self) -> u8 {
        self.id
    }
}

impl PlayerState {
    pub fn new(id: u8) -> Self {
        PlayerState {
            id,
            deck: CardList::default(),
            hand: CardList::default(),
            graveyard: CardList::default(),
            field: CardList::default(),
            shards: ShardList::new(),
            stats: PlayerStats::default(),
            counters: PlayerCounters::default(),
            endgame: None,
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
                state.debug.flags.contains(DebugFlags::IGNORE_COST)
                    || item.card.computed().cost.value() == 0
                    || self.shards.get(item.card.computed().color)
                        >= item.card.computed().cost.value() as u32
            })
            .filter(|item| {
                !item.card.computed().is_creature()
                    || item.card.computed().cost.value() > 0
                    || self.counters.free_casted == 0
            })
            .filter(|item| item.card.effect().is_castable(state, &item.card))
            .map(|item| item.card.id())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Encode, Decode)]
pub struct PlayerStats {
    pub life: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self { life: 2000 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum PlayerEndgameState {
    Win(EndgameReason),
    Lose(EndgameReason),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PlayerCounters {
    pub draw: u32,
    pub free_casted: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct PlayerZone {
    pub player: u8,
    pub zone: Zone,
}

impl PlayerZone {
    pub fn new(player: u8, zone: Zone) -> Self {
        Self { player, zone }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct LocalPlayerState {
    pub id: u8,
    pub deck: usize,
    pub hand: Vec<HandItem<CardSnapshot>>,
    pub graveyard: Vec<CardSnapshot>,
    pub field: Vec<FieldItem<CardSnapshot>>,
    pub shards: ShardList,
    pub stats: PlayerStats,
}

impl PlayerItem for LocalPlayerState {
    fn id(&self) -> u8 {
        self.id
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LocalStateAccess {
    PublicOnly,
    Player(u8),
    Full,
}

impl LocalPlayerState {
    pub fn new(state: &PlayerState, access: LocalStateAccess) -> Self {
        let private = match access {
            LocalStateAccess::PublicOnly => false,
            LocalStateAccess::Player(player) => player == state.id,
            LocalStateAccess::Full => true,
        };
        Self {
            id: state.id,
            deck: state.deck.len(),
            hand: state
                .hand
                .items()
                .map(|item| {
                    let revealed = item.card.revealed().contains(state.id);
                    let card = item.card.snapshot();
                    let card = if private || revealed {
                        card
                    } else {
                        card.redacted()
                    };
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(transparent)]
pub struct PlayerMask(u8);

impl PlayerMask {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set(&mut self, player: u8, value: bool) {
        if value {
            self.0 |= 1 << player;
        } else {
            self.0 &= !(1 << player);
        }
    }

    pub fn contains(&self, player: u8) -> bool {
        self.0 & (1 << player) != 0
    }

    pub fn iter(self) -> impl Iterator<Item = u8> {
        (0..8).filter(move |&player| self.contains(player))
    }
}
