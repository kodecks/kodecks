use crate::{
    action::{Action, PlayerAvailableActions},
    card::{ArchetypeId, Card, Catalog},
    computed::ComputedSequence,
    continuous::ContinuousEffectList,
    effect::EffectTriggerContext,
    error::Error,
    filter_vec,
    id::{ObjectId, ObjectIdCounter},
    log::LogAction,
    opcode::OpcodeList,
    phase::Phase,
    player::{PlayerEndgameState, PlayerList, PlayerState, PlayerZone},
    profile::DebugFlags,
    profile::GameProfile,
    sequence::CardSequence,
    stack::{Stack, StackItem},
    zone::{CardZone, Zone},
};
use bincode::{Decode, Encode};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt};
use tracing::{error, warn};

mod action;
mod event;
mod local;
mod opcode;
mod phase;
mod state;

pub use local::LocalEnvironment;
pub use state::*;

#[derive(Clone)]
pub struct Environment {
    pub state: GameState,
    opcodes: VecDeque<OpcodeList>,
    stack: Stack<StackItem>,
    continuous: ContinuousEffectList,
    endgame: EndgameState,
    timestamp: u64,
    last_available_actions: Option<PlayerAvailableActions>,
    rng: SmallRng,
    catalog: &'static Catalog,
    obj_counter: ObjectIdCounter,
}

impl Environment {
    pub fn new(profile: GameProfile, catalog: &'static Catalog) -> Self {
        let mut rng: SmallRng = profile
            .debug
            .rng_seed
            .map(SmallRng::seed_from_u64)
            .unwrap_or_else(SmallRng::from_entropy);

        let mut obj_counter = ObjectIdCounter::default();
        let mut players = profile
            .players
            .into_iter()
            .map(|player| {
                let mut state = PlayerState::new(player.id);
                for item in &player.deck.cards {
                    let archetype = &catalog[item.archetype_id];
                    let card = Card::new(&mut obj_counter, item, archetype, player.id);
                    state.deck.add_top(card);
                }
                if !profile.debug.no_deck_shuffle {
                    state.deck.shuffle(&mut obj_counter, &mut rng);
                }
                state
            })
            .collect::<Vec<_>>();

        if !profile.debug.no_player_shuffle {
            players.shuffle(&mut rng);
        }

        let current_player = players.first().as_ref().unwrap().id;

        Environment {
            state: GameState {
                regulation: profile.regulation,
                debug: profile.debug,
                turn: 0,
                phase: Phase::Standby,
                players: PlayerList::new(current_player, players),
            },
            opcodes: VecDeque::new(),
            stack: Stack::new(),
            continuous: Default::default(),
            endgame: EndgameState::InProgress,
            timestamp: 0,
            last_available_actions: None,
            rng,
            catalog,
            obj_counter,
        }
    }

    fn compute_effects(&mut self) -> Result<(), Error> {
        let sides = self
            .state
            .players
            .iter()
            .map(|player| player.id)
            .collect::<Vec<_>>();
        for side in sides {
            let abilities = self.continuous.apply_player(&self.state, side);
            let player = self.state.players.get(side);
            let field_states = player
                .field
                .iter()
                .map(|card| self.continuous.apply_card(&self.state, card))
                .collect();
            let hand_states = player
                .hand
                .items()
                .map(|item| {
                    let mut attrs = self.continuous.apply_card(&self.state, &item.card);
                    attrs.cost.add(item.cost_delta);
                    attrs
                })
                .collect();
            let player = self.state.players.get_mut(side);
            player.abilities = abilities;
            player.field.update_computed(field_states);
            player.hand.update_computed(hand_states);
        }
        Ok(())
    }

    pub fn process(&mut self, player: u8, action: Option<Action>) -> Report {
        let report = match (&self.last_available_actions, action.clone()) {
            (_, Some(Action::Concede)) => self.process_turn(player, Some(Action::Concede)),
            (None, _) => self.process_turn(player, None),
            (Some(available), Some(action)) if available.validate(player, &action) => {
                self.process_turn(player, Some(action))
            }
            _ => {
                warn!("Invalid action: {:?} for player: {}", action, player);
                Report {
                    available_actions: self.last_available_actions.clone(),
                    logs: vec![],
                    endgame: self.endgame,
                    timestamp: self.timestamp,
                }
            }
        };
        self.last_available_actions = report.available_actions.clone();
        report
    }

    fn process_turn(&mut self, player: u8, mut action: Option<Action>) -> Report {
        let action = match action.take() {
            Some(Action::Concede) => {
                let loser = self.state.players.get_mut(player);
                loser.endgame = Some(PlayerEndgameState::Lose(EndgameReason::Concede));
                Some(Action::Concede)
            }
            Some(Action::DebugCommand { commands })
                if self.state.debug.flags.contains(DebugFlags::DEBUG_COMMAND) =>
            {
                for command in commands {
                    match command.into_opcodes(self) {
                        Ok(log) => self
                            .opcodes
                            .extend(log.into_iter().filter(|item| !item.is_empty())),
                        Err(err) => {
                            error!("Error processing command: {:?}", err);
                        }
                    }
                }
                None
            }
            other => other,
        };

        if let Some(item) = self.stack.pop() {
            let card = self.state.find_card(item.source).unwrap();
            let mut ctx = EffectTriggerContext::new(&self.state, &mut self.obj_counter, card);

            let targeted = match &action {
                Some(Action::SelectCard { card }) => Some(LogAction::CardsTargeted {
                    source: item.source,
                    targets: vec![*card],
                }),
                _ => None,
            };

            match (item.handler)(&mut ctx, action) {
                Ok(report) => {
                    let (continuous, _) = ctx.into_inner();
                    self.continuous.extend(continuous);

                    let mut list = vec![];
                    for command in report.commands {
                        match command.into_opcodes(self) {
                            Ok(codes) => {
                                list.extend(codes.into_iter().filter(|item| !item.is_empty()))
                            }
                            Err(err) => {
                                error!("Error processing command: {:?}", err);
                            }
                        }
                    }

                    let mut logs = filter_vec![targeted,];

                    for item in list {
                        for opcode in item {
                            match self.execute(opcode) {
                                Ok(log) => logs.extend(log),
                                Err(err) => {
                                    error!("Error executing opcode: {:?}", err);
                                }
                            }
                        }
                    }

                    self.continuous.update(&self.state);
                    if let Err(err) = self.compute_effects() {
                        error!("Error computing effects: {:?}", err);
                    }

                    self.check_game_condition();

                    if !report
                        .available_actions
                        .as_ref()
                        .map_or(true, |item| item.actions.is_empty())
                    {
                        self.stack.push(item);
                    }

                    return Report {
                        available_actions: report.available_actions,
                        logs,
                        endgame: self.endgame,
                        timestamp: self.timestamp,
                    };
                }
                Err(err) => {
                    error!("Error processing stack item: {:?}", err);
                }
            }
        } else if self.opcodes.is_empty() {
            let mut phase = self.state.phase.clone();
            let opcodes = match self.process_player_phase(action, &mut phase) {
                Ok(opcodes) => opcodes,
                Err(err) => {
                    error!("Error processing player phase: {:?}", err);
                    vec![]
                }
            };
            self.opcodes
                .extend(opcodes.into_iter().filter(|item| !item.is_empty()));
            self.state.phase = phase;
        }

        let next = self.opcodes.pop_front();
        let next_empty = self.opcodes.is_empty();

        let mut logs = vec![];
        if let Some(log) = next {
            for opcode in log {
                match self.execute(opcode) {
                    Ok(log) => logs.extend(log),
                    Err(err) => {
                        error!("Error executing opcode: {:?}", err);
                    }
                }
            }
        }

        self.continuous.update(&self.state);
        if let Err(err) = self.compute_effects() {
            error!("Error computing effects: {:?}", err);
        }

        self.check_game_condition();

        let available_actions = if next_empty {
            self.available_actions()
        } else {
            None
        };

        Report {
            available_actions,
            logs,
            endgame: self.endgame,
            timestamp: self.timestamp,
        }
    }

    pub fn last_available_actions(&self) -> Option<&PlayerAvailableActions> {
        self.last_available_actions.as_ref()
    }

    pub fn game_condition(&self) -> EndgameState {
        self.endgame
    }

    pub fn check_game_condition(&mut self) {
        if self.endgame.is_ended() {
            return;
        }

        for player in self
            .state
            .players
            .iter_mut()
            .filter(|player| player.endgame.is_none())
        {
            if player.stats.life == 0 {
                player.endgame = Some(PlayerEndgameState::Lose(EndgameReason::LifeZero));
            }
        }

        let won_players = self
            .state
            .players
            .iter()
            .filter_map(|player| match player.endgame {
                Some(PlayerEndgameState::Win(reason)) => Some((player, reason)),
                _ => None,
            })
            .collect::<Vec<_>>();

        let lost_players = self
            .state
            .players
            .iter()
            .filter_map(|player| match player.endgame {
                Some(PlayerEndgameState::Lose(reason)) => Some((player, reason)),
                _ => None,
            })
            .collect::<Vec<_>>();

        self.endgame = if won_players.is_empty() && lost_players.is_empty() {
            EndgameState::InProgress
        } else if let [(won, reason)] = won_players.as_slice() {
            EndgameState::Finished {
                winner: Some(won.id),
                reason: *reason,
            }
        } else if let [(lost, reason)] = lost_players.as_slice() {
            EndgameState::Finished {
                winner: Some(self.state.players.next_id(lost.id)),
                reason: *reason,
            }
        } else {
            EndgameState::Finished {
                winner: None,
                reason: EndgameReason::SimultaneousEnd,
            }
        };
    }

    pub fn generate_card_token(&self, player: u8, token: ObjectId, archetype: ArchetypeId) -> Card {
        let archetype = &self.catalog[archetype];
        let mut card = Card::new_token(token, archetype, player);
        card.set_timestamp(self.timestamp);
        card.set_zone(PlayerZone::new(player, Zone::Field));
        card
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum EndgameState {
    InProgress,
    Finished {
        winner: Option<u8>,
        reason: EndgameReason,
    },
}

impl EndgameState {
    pub fn is_ended(&self) -> bool {
        !matches!(self, EndgameState::InProgress)
    }
}

impl fmt::Display for EndgameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EndgameState::InProgress => write!(f, "In Progress"),
            EndgameState::Finished { winner, .. } => {
                if let Some(winner) = winner {
                    write!(f, "Winner: {}", winner)
                } else {
                    write!(f, "Draw")
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum EndgameReason {
    Concede,
    LifeZero,
    DeckEmpty,
    SimultaneousEnd,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub available_actions: Option<PlayerAvailableActions>,
    pub logs: Vec<LogAction>,
    pub endgame: EndgameState,
    pub timestamp: u64,
}
