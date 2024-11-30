use crate::{
    ability::{AbilityList, PlayerAbility},
    card::{Card, CardScore, CardSnapshot},
    color::Color,
    deck::DeckList,
    env::{EndgameReason, GameState},
    error::ActionError,
    id::{CardId, ObjectId, TimedCardId, TimedObjectId},
    list::CardList,
    profile::DebugFlags,
    score::Score,
    shard::ShardList,
    slot::CardSlot,
    zone::{CardZone, ZoneKind},
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
    pub fn get(&self, player: u8) -> Result<&T, ActionError> {
        self.players
            .iter()
            .find(|item| item.id() == player)
            .ok_or(ActionError::PlayerNotFound { player })
    }

    pub fn get_mut(&mut self, player: u8) -> Result<&mut T, ActionError> {
        let index = self
            .players
            .iter()
            .position(|item| item.id() == player)
            .ok_or(ActionError::PlayerNotFound { player })?;
        Ok(&mut self.players[index])
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        PlayerListIter::new(self.player_in_turn, &self.players)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        PlayerListMutIter::new(self.player_in_turn, &mut self.players)
    }

    pub fn next_id(&self, id: u8) -> Result<u8, ActionError> {
        if self.players.is_empty() {
            return Err(ActionError::PlayerNotFound { player: id });
        }
        let pos = self
            .players
            .iter()
            .position(|player| player.id() == id)
            .unwrap_or(0);
        let next = (pos + 1) % self.players.len();
        Ok(self.players[next].id())
    }

    pub fn next_player(&self, id: u8) -> Result<&T, ActionError> {
        let next = self.next_id(id)?;
        self.get(next)
    }

    pub fn push(&mut self, player: T) {
        self.players.push(player);
    }

    pub fn player_in_turn(&self) -> Result<&T, ActionError> {
        self.get(self.player_in_turn)
    }
}

pub trait PlayerItem {
    fn id(&self) -> u8;
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u8,
    pub deck: CardList<Card>,
    pub hand: CardList<Card>,
    pub graveyard: CardList<Card>,
    pub field: CardSlot<Card>,
    pub limbo: CardList<Card>,
    pub shards: ShardList,
    pub stats: PlayerStats,
    pub counters: PlayerCounters,
    pub endgame: Option<PlayerEndgameState>,
    pub abilities: AbilityList<PlayerAbility>,
}

impl PlayerItem for Player {
    fn id(&self) -> u8 {
        self.id
    }
}

impl Player {
    pub fn new(id: u8) -> Self {
        Player {
            id,
            deck: CardList::default(),
            hand: CardList::default(),
            graveyard: CardList::default(),
            field: CardSlot::new(3),
            limbo: CardList::default(),
            shards: ShardList::new(),
            stats: PlayerStats::default(),
            counters: PlayerCounters::default(),
            endgame: None,
            abilities: AbilityList::default(),
        }
    }

    pub fn find_card<T>(&self, card: T) -> Option<&Card>
    where
        T: CardId + Copy,
    {
        self.deck
            .get(card)
            .or_else(|| self.hand.get(card))
            .or_else(|| self.graveyard.get(card))
            .or_else(|| self.field.get(card))
            .or_else(|| self.limbo.get(card))
    }

    pub fn find_card_mut(&mut self, card: ObjectId) -> Option<&mut Card> {
        self.deck
            .get_mut(card)
            .or_else(|| self.hand.get_mut(card))
            .or_else(|| self.graveyard.get_mut(card))
            .or_else(|| self.field.get_mut(card))
            .or_else(|| self.limbo.get_mut(card))
    }

    pub fn find_zone(&self, card: ObjectId) -> Option<ZoneKind> {
        if self.deck.contains(card) {
            Some(ZoneKind::Deck)
        } else if self.hand.contains(card) {
            Some(ZoneKind::Hand)
        } else if self.field.contains(card) {
            Some(ZoneKind::Field)
        } else if self.graveyard.contains(card) {
            Some(ZoneKind::Graveyard)
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
    ) -> impl Iterator<Item = TimedObjectId> + 'a {
        self.hand
            .iter()
            .filter(|card| {
                let castable = state.debug.flags.contains(DebugFlags::IGNORE_COST)
                    || card.computed().cost.value() == 0
                    || self.shards.get(Color::COLORLESS) >= card.computed().cost.value();
                card.effect().is_castable(state, card, castable)
            })
            .filter(|_| self.field.has_space())
            .map(|card| card.timed_id())
    }
}

#[derive(Debug, Clone)]
pub struct PlayerScore {
    pub id: u8,
    pub life: i32,
    pub shards: i32,
    pub hand: Vec<CardScore>,
    pub field: Vec<CardScore>,
}

impl Score for PlayerScore {
    type Output = i32;

    fn score(&self) -> i32 {
        self.life
            + self.shards
            + self.hand.iter().map(Score::score).sum::<i32>()
            + self.field.iter().map(Score::score).sum::<i32>()
    }
}

impl Score for Player {
    type Output = PlayerScore;

    fn score(&self) -> PlayerScore {
        PlayerScore {
            id: self.id,
            life: self.stats.life as i32,
            shards: self.shards.len() as i32,
            hand: self.hand.iter().map(Score::score).collect(),
            field: self.field.iter().map(Score::score).collect(),
        }
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
    pub draw: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct Zone {
    pub player: u8,
    pub kind: ZoneKind,
}

impl Zone {
    pub fn new(player: u8, kind: ZoneKind) -> Self {
        Self { player, kind }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct LocalPlayerState {
    pub id: u8,
    pub deck: usize,
    pub hand: Vec<CardSnapshot>,
    pub graveyard: Vec<CardSnapshot>,
    pub field: Vec<CardSnapshot>,
    pub limbo: Vec<CardSnapshot>,
    pub shards: ShardList,
    pub stats: PlayerStats,
}

impl PlayerItem for LocalPlayerState {
    fn id(&self) -> u8 {
        self.id
    }
}

impl LocalPlayerState {
    pub fn new(state: &Player, viewer: u8) -> Self {
        Self {
            id: state.id,
            deck: state.deck.len(),
            hand: state
                .hand
                .iter()
                .map(|card| card.snapshot().redacted(viewer))
                .collect(),
            graveyard: state
                .graveyard
                .iter()
                .map(|card| card.snapshot().redacted(viewer))
                .collect(),
            field: state
                .field
                .iter()
                .map(|card| card.snapshot().redacted(viewer))
                .collect(),
            limbo: state
                .limbo
                .iter()
                .map(|card| card.snapshot().redacted(viewer))
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
            .or_else(|| self.limbo.iter().find(|item| item.id == card))
    }

    pub fn cards(&self) -> impl Iterator<Item = &CardSnapshot> {
        self.hand
            .iter()
            .chain(self.graveyard.iter())
            .chain(self.field.iter())
            .chain(self.limbo.iter())
    }

    pub fn find_zone(&self, card: ObjectId) -> Option<ZoneKind> {
        if self.hand.iter().any(|item| item.id == card) {
            return Some(ZoneKind::Hand);
        }
        if self.field.iter().any(|item| item.id == card) {
            return Some(ZoneKind::Field);
        }
        if self.graveyard.iter().any(|item| item.id == card) {
            return Some(ZoneKind::Graveyard);
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

    fn remove<T>(&mut self, id: T) -> Option<Self::Item>
    where
        T: CardId,
    {
        let index = self.iter().position(|card| card.id == id.id())?;
        Some(self.remove(index))
    }

    fn get<T>(&self, id: T) -> Option<&Self::Item>
    where
        T: CardId,
    {
        self.iter().find(|card| card.id == id.id())
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = &Self::Item> {
        (self as &[CardSnapshot]).iter()
    }

    fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Self::Item> {
        (self as &mut [CardSnapshot]).iter_mut()
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

    pub fn set_all(&mut self, value: bool) {
        self.0 = if value { 0xff } else { 0 };
    }
}
