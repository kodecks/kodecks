use crate::{
    action::{Action, PlayerAvailableActions},
    archetype::ArchetypeId,
    card::Card,
    catalog::Catalog,
    computed::ComputedSequence,
    continuous::ContinuousEffectList,
    effect::EffectTriggerContext,
    error::ActionError,
    filter_vec,
    id::{ObjectId, ObjectIdCounter},
    log::GameLog,
    opcode::OpcodeList,
    phase::Phase,
    player::{Player, PlayerEndgameState, PlayerList, Zone},
    profile::{DebugFlags, GameProfile},
    sequence::CardSequence,
    stack::{Stack, StackItem},
    zone::{CardZone, ZoneKind},
};
use bincode::{Decode, Encode};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt, sync::Arc};
use strum::Display;
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
    timestamp: u32,
    last_available_actions: Option<PlayerAvailableActions>,
    rng: SmallRng,
    catalog: Arc<Catalog>,
    obj_counter: ObjectIdCounter,
}

impl Environment {
    pub fn new(profile: GameProfile, catalog: Arc<Catalog>) -> Self {
        let debug = profile.debug.unwrap_or_default();
        let mut rng: SmallRng = profile
            .rng_seed
            .map(SmallRng::seed_from_u64)
            .unwrap_or_else(SmallRng::from_entropy);

        let mut obj_counter = ObjectIdCounter::default();
        let players = profile
            .players
            .into_iter()
            .enumerate()
            .map(|(id, player)| {
                let mut state = Player::new(id as u8);
                for item in &player.deck.cards {
                    let archetype = &catalog[item.card.archetype_id];
                    let card = Card::new(
                        &mut obj_counter,
                        item,
                        archetype.clone(),
                        item.card.style,
                        id as u8,
                    );
                    state.deck.add_top(card);
                }
                state
            })
            .collect::<Vec<_>>();

        let current_player = if debug.no_player_shuffle {
            players.first().as_ref().unwrap().id
        } else {
            players.choose(&mut rng).unwrap().id
        };

        Environment {
            state: GameState {
                regulation: profile.regulation,
                debug,
                turn: 0,
                phase: Phase::Standby,
                players: PlayerList::new(current_player, players),
                endgame: EndgameState::InProgress,
            },
            opcodes: VecDeque::new(),
            stack: Stack::new(),
            continuous: Default::default(),
            timestamp: 0,
            last_available_actions: None,
            rng,
            catalog,
            obj_counter,
        }
    }

    fn compute_effects(&mut self) -> Result<(), ActionError> {
        let sides = self
            .state
            .players
            .iter()
            .map(|player| player.id)
            .collect::<Vec<_>>();
        for side in sides {
            let abilities = self.continuous.apply_player(&self.state, side);
            let player = self.state.players.get(side)?;
            let field_states = player
                .field
                .iter()
                .map(|card| self.continuous.apply_card(&self.state, card))
                .collect();
            let hand_states = player
                .hand
                .iter()
                .map(|card| self.continuous.apply_card(&self.state, card))
                .collect();
            let player = self.state.players.get_mut(side)?;
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
                    endgame: self.state.endgame,
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
                if let Ok(loser) = self.state.players.get_mut(player) {
                    loser.endgame = Some(PlayerEndgameState::Lose(EndgameReason::Concede));
                }
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
            let source = self.state.find_card(item.source).unwrap();
            let mut ctx = EffectTriggerContext::new(&self.state, &mut self.obj_counter, source);

            let targeted = match &action {
                Some(Action::SelectCard { card }) => {
                    let target = self.state.find_card(*card).unwrap();
                    Some(GameLog::CardTargeted {
                        source: source.snapshot(),
                        target: target.snapshot(),
                    })
                }
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
                            match self.execute(opcode.clone()) {
                                Ok(log) => logs.extend(log),
                                Err(err) => {
                                    error!("Error executing opcode: {:?} {:?}", err, opcode);
                                }
                            }
                        }
                    }

                    self.continuous.update();
                    if let Err(err) = self.compute_effects() {
                        error!("Error computing effects: {:?}", err);
                    }

                    if self.check_game_condition() {
                        if let EndgameState::Finished { winner, reason } = self.state.endgame {
                            logs.push(GameLog::GameEnded { winner, reason });
                        }
                    }

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
                        endgame: self.state.endgame,
                        timestamp: self.timestamp,
                    };
                }
                Err(err) => {
                    error!("Error processing stack item: {:?}", err);
                }
            }
        } else if self.opcodes.is_empty() {
            let opcodes = match self.process_player_phase(action) {
                Ok(opcodes) => opcodes,
                Err(err) => {
                    error!("Error processing player phase: {:?}", err);
                    vec![]
                }
            };
            self.opcodes
                .extend(opcodes.into_iter().filter(|item| !item.is_empty()));
        }

        let next = self.opcodes.pop_front();
        let next_empty = self.opcodes.is_empty();

        let mut logs = vec![];
        if let Some(log) = next {
            for opcode in log {
                match self.execute(opcode.clone()) {
                    Ok(log) => logs.extend(log),
                    Err(err) => {
                        error!("Error executing opcode: {:?} {:?}", err, opcode);
                    }
                }
            }
        }

        self.continuous.update();
        if let Err(err) = self.compute_effects() {
            error!("Error computing effects: {:?}", err);
        }

        if self.check_game_condition() {
            if let EndgameState::Finished { winner, reason } = self.state.endgame {
                logs.push(GameLog::GameEnded { winner, reason });
            }
        }

        let available_actions = if next_empty {
            self.available_actions()
        } else {
            None
        };

        Report {
            available_actions,
            logs,
            endgame: self.state.endgame,
            timestamp: self.timestamp,
        }
    }

    pub fn last_available_actions(&self) -> Option<&PlayerAvailableActions> {
        self.last_available_actions.as_ref()
    }

    pub fn game_condition(&self) -> EndgameState {
        self.state.endgame
    }

    pub fn check_game_condition(&mut self) -> bool {
        if self.state.endgame.is_ended() {
            return false;
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

        let new_condition = if won_players.is_empty() && lost_players.is_empty() {
            EndgameState::InProgress
        } else if let [(won, reason)] = won_players.as_slice() {
            EndgameState::Finished {
                winner: Some(won.id),
                reason: *reason,
            }
        } else if let [(lost, reason)] = lost_players.as_slice() {
            EndgameState::Finished {
                winner: self.state.players.next_id(lost.id).ok(),
                reason: *reason,
            }
        } else {
            EndgameState::Finished {
                winner: None,
                reason: EndgameReason::SimultaneousEnd,
            }
        };
        if self.state.endgame != new_condition {
            self.state.endgame = new_condition;
            true
        } else {
            false
        }
    }

    pub fn generate_card_token(&self, player: u8, token: ObjectId, archetype: ArchetypeId) -> Card {
        let archetype = &self.catalog[archetype];
        let mut card = Card::new_token(token, archetype.clone(), player);
        card.set_zone(Zone::new(player, ZoneKind::Field));
        card
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum EndgameReason {
    Concede,
    LifeZero,
    DeckOut,
    SimultaneousEnd,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub available_actions: Option<PlayerAvailableActions>,
    pub logs: Vec<GameLog>,
    pub endgame: EndgameState,
    pub timestamp: u32,
}
