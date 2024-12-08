use super::Environment;
use crate::{
    ability::KeywordAbility,
    action::Action,
    command::ActionCommand,
    error::ActionError,
    event::{CardEvent, EventReason},
    field::{FieldBattleState, FieldState},
    filter_vec,
    id::TimedCardId,
    opcode::{Opcode, OpcodeList},
    phase::Phase,
    player::Zone,
    profile::DebugFlags,
    target::Target,
    zone::{CardZone, MoveReason, ZoneKind},
};
use std::{iter, vec};

impl Environment {
    fn initialize(&self) -> Result<Vec<OpcodeList>, ActionError> {
        let initial_life = self.state.regulation.initial_life;

        Ok(filter_vec![
            Some(OpcodeList::new(vec![Opcode::StartGame],)),
            self.state.players.iter().flat_map(|player| {
                filter_vec![Some(OpcodeList::new(vec![Opcode::SetLife {
                    player: player.id,
                    life: initial_life,
                }])),]
            }),
            if self.state.debug.no_deck_shuffle {
                vec![]
            } else {
                self.state
                    .players
                    .iter()
                    .flat_map(|player| {
                        filter_vec![Some(OpcodeList::new(vec![Opcode::ShuffleDeck {
                            player: player.id,
                        }])),]
                    })
                    .collect::<Vec<_>>()
            },
            self.state.players.iter().flat_map(|player| {
                let draw = iter::repeat(OpcodeList::new(vec![Opcode::DrawCard {
                    player: player.id,
                }]))
                .take(self.state.regulation.initial_hand_size as usize);
                let fetch = iter::repeat(OpcodeList::new(vec![Opcode::LoadCard {
                    player: player.id,
                }]))
                .take(3);
                filter_vec![draw, fetch,]
            }),
            Some(OpcodeList::new(vec![Opcode::ChangeTurn {
                turn: 1,
                player: self.state.players.player_in_turn()?.id,
                phase: self.state.phase,
            },],)),
        ])
    }

    pub fn process_player_phase(
        &self,
        action: Option<Action>,
    ) -> Result<Vec<OpcodeList>, ActionError> {
        if self.state.turn == 0 {
            return self.initialize();
        }

        let used_hexes = self
            .state
            .players
            .iter()
            .flat_map(|player| {
                player
                    .field
                    .iter()
                    .filter(|card| {
                        card.computed().is_hex()
                            && !self.stack.iter().any(|stack| stack.source == card.id())
                    })
                    .map(|card| card.id())
            })
            .collect::<Vec<_>>();

        if !used_hexes.is_empty() {
            return Ok(used_hexes
                .into_iter()
                .map(|card| {
                    let card = self.state.find_card(card).unwrap();
                    let zone = card.zone();
                    OpcodeList::new(vec![Opcode::MoveCard {
                        card: card.id(),
                        from: *zone,
                        to: Zone::new(card.owner(), ZoneKind::Graveyard),
                        reason: MoveReason::Move,
                    }])
                })
                .collect());
        }

        let player_in_turn = &self
            .state
            .players
            .get(self.state.players.player_in_turn()?.id)?;
        match self.state.phase {
            Phase::Standby => {
                let next_phase = Phase::Draw;
                Ok(filter_vec![Some(OpcodeList::new(vec![
                    Opcode::ResetPlayerState {
                        player: player_in_turn.id,
                    },
                    Opcode::ChangePhase { phase: next_phase }
                ],)),])
            }
            Phase::Draw => Ok(vec![OpcodeList::new(vec![Opcode::ChangePhase {
                phase: Phase::Main,
            }])]),
            Phase::Main => {
                let logs = match action {
                    Some(Action::FetchCard { card }) => {
                        let card = player_in_turn.colony.get(card).unwrap();
                        return Ok(vec![OpcodeList::new(vec![Opcode::FetchCard {
                            player: player_in_turn.id,
                            card: card.id(),
                        }])]);
                    }
                    Some(Action::CastCard { card }) => {
                        let item = player_in_turn
                            .hand
                            .get(card)
                            .ok_or(ActionError::CardNotFound { id: card.id })?;
                        let color = item.computed().color;
                        let cost = if self.state.debug.flags.contains(DebugFlags::IGNORE_COST) {
                            0
                        } else {
                            item.computed().cost.value()
                        };
                        if player_in_turn.stats.manas() < cost {
                            return Err(ActionError::InsufficientShards {
                                color,
                                amount: cost,
                            });
                        }
                        let from = Zone::new(player_in_turn.id, ZoneKind::Hand);
                        filter_vec![
                            Some(OpcodeList::new(filter_vec![
                                if cost > 0 {
                                    Some(Opcode::ConsumeManas {
                                        player: self.state.players.player_in_turn()?.id,
                                        amount: cost,
                                    })
                                } else {
                                    None
                                },
                                Some(Opcode::CastCard {
                                    player: self.state.players.player_in_turn()?.id,
                                    card: item.id(),
                                    cost
                                }),
                            ],)),
                            self.apply_event(CardEvent::Casted { from }, item, item)
                                .ok()
                                .into_iter()
                                .flatten(),
                            self.apply_event_any(CardEvent::AnyCasted, item)
                                .ok()
                                .into_iter()
                                .flatten(),
                        ]
                    }
                    Some(Action::Attack { attackers }) => {
                        let attackers = attackers
                            .iter()
                            .map(|id| player_in_turn.field.get(*id).unwrap())
                            .collect::<Vec<_>>();
                        let opcodes = attackers
                            .iter()
                            .flat_map(|card| {
                                vec![Opcode::SetBattleState {
                                    card: card.id(),
                                    state: Some(FieldBattleState::Attacking),
                                }]
                            })
                            .collect::<Vec<_>>();
                        vec![
                            OpcodeList::new(opcodes),
                            OpcodeList::new(vec![Opcode::ChangePhase {
                                phase: Phase::Block,
                            }]),
                        ]
                    }
                    Some(Action::EndTurn) => {
                        vec![OpcodeList::new(vec![
                            Opcode::ResetBattleState,
                            Opcode::ChangePhase { phase: Phase::End },
                        ])]
                    }
                    _ => vec![],
                };
                Ok(logs)
            }
            Phase::Block => {
                let active_player = &self
                    .state
                    .players
                    .next_player(self.state.players.player_in_turn()?.id)?;

                if player_in_turn.field.attacking_cards().next().is_none() {
                    Ok(vec![OpcodeList::new(filter_vec![Some(
                        Opcode::ChangePhase {
                            phase: Phase::Battle,
                        }
                    ),])])
                } else if let Some(Action::Block { pairs }) = action {
                    Ok(vec![OpcodeList::new(filter_vec![
                        pairs.iter().flat_map(|(attacker, blocker)| {
                            vec![Opcode::SetBattleState {
                                card: blocker.id,
                                state: Some(FieldBattleState::Blocking {
                                    attacker: *attacker,
                                }),
                            }]
                        }),
                        Some(Opcode::ChangePhase {
                            phase: Phase::Battle,
                        }),
                    ])])
                } else if let Some(Action::CastCard { card }) = action {
                    let item = active_player
                        .hand
                        .get(card)
                        .ok_or(ActionError::CardNotFound { id: card.id })?;
                    let color = item.computed().color;
                    let cost = if self.state.debug.flags.contains(DebugFlags::IGNORE_COST) {
                        0
                    } else {
                        item.computed().cost.value()
                    };
                    if active_player.stats.manas() < cost {
                        return Err(ActionError::InsufficientShards {
                            color,
                            amount: cost,
                        });
                    }
                    let from = Zone::new(active_player.id, ZoneKind::Hand);
                    Ok(filter_vec![
                        Some(OpcodeList::new(filter_vec![
                            if cost > 0 {
                                Some(Opcode::ConsumeManas {
                                    player: active_player.id,
                                    amount: cost,
                                })
                            } else {
                                None
                            },
                            Some(Opcode::CastCard {
                                player: active_player.id,
                                card: item.id(),
                                cost
                            }),
                        ],)),
                        self.apply_event(CardEvent::Casted { from }, item, item)
                            .ok()
                            .into_iter()
                            .flatten(),
                        self.apply_event_any(CardEvent::AnyCasted, item)
                            .ok()
                            .into_iter()
                            .flatten(),
                    ])
                } else {
                    Ok(vec![])
                }
            }
            Phase::Battle => {
                let target = &self
                    .state
                    .players
                    .next_player(self.state.players.player_in_turn()?.id)?;
                let attacker = player_in_turn.field.attacking_cards().min_by_key(|card| {
                    (
                        if target.field.find_blocker(card.id()).is_some() {
                            0
                        } else {
                            1
                        },
                        card.timestamp(),
                    )
                });

                if let Some(attacker) = attacker {
                    let blocker = target.field.find_blocker(attacker.id());
                    let attacker_power = attacker.computed().power.unwrap_or_default().value();

                    let mut logs = vec![];
                    if let Ok(log) = self.apply_event(CardEvent::Attacking, attacker, attacker) {
                        logs.extend(log);
                    }

                    logs.push(OpcodeList::new(vec![
                        Opcode::Attack {
                            attacker: attacker.id(),
                            target: if let Some(blocker) = blocker {
                                Target::Card(blocker.id())
                            } else {
                                Target::Player(target.id)
                            },
                        },
                        Opcode::SetBattleState {
                            card: attacker.id(),
                            state: Some(FieldBattleState::Attacked),
                        },
                    ]));

                    if let Ok(log) = (ActionCommand::SetFieldState {
                        source: attacker.id(),
                        target: attacker.timed_id(),
                        state: FieldState::Exhausted,
                        reason: EventReason::Battle,
                    })
                    .into_opcodes(self)
                    {
                        logs.extend(log);
                    }

                    if blocker.is_none() && attacker_power > 0 {
                        logs.push(OpcodeList::new(vec![Opcode::InflictDamage {
                            player: target.id,
                            amount: attacker_power,
                        }]));

                        if let Ok(log) = self.apply_event(
                            CardEvent::DealtDamage {
                                player: target.id,
                                amount: attacker_power,
                                reason: EventReason::Battle,
                            },
                            attacker,
                            attacker,
                        ) {
                            logs.extend(log);
                        }
                    }

                    if let Some(blocker) = blocker {
                        if let Ok(log) = self.apply_event(CardEvent::Blocking, blocker, blocker) {
                            logs.extend(log);
                        }

                        let mut destroyed_logs = vec![];
                        let blocker_has_toxic = blocker
                            .computed()
                            .abilities
                            .contains(&KeywordAbility::Toxic);
                        let blocker_power = blocker.computed().power.unwrap_or_default().value();
                        if (blocker_power > 0 && attacker_power <= blocker_power)
                            || blocker_has_toxic
                        {
                            if let Ok(log) = (ActionCommand::DestroyCard {
                                source: blocker.id(),
                                target: attacker.timed_id(),
                                reason: EventReason::Battle,
                            })
                            .into_opcodes(self)
                            {
                                destroyed_logs.extend(log.into_iter().flatten());
                            }
                        }

                        let attacker_has_toxic = attacker
                            .computed()
                            .abilities
                            .contains(&KeywordAbility::Toxic);
                        if (attacker_power > 0 && blocker_power <= attacker_power)
                            || attacker_has_toxic
                        {
                            if let Ok(log) = (ActionCommand::DestroyCard {
                                source: attacker.id(),
                                target: blocker.timed_id(),
                                reason: EventReason::Battle,
                            })
                            .into_opcodes(self)
                            {
                                destroyed_logs.extend(log.into_iter().flatten());
                            }
                        }

                        if !destroyed_logs.is_empty() {
                            logs.push(OpcodeList::new(destroyed_logs));
                        }
                    }

                    if let Ok(log) = self.apply_event(CardEvent::Attacked, attacker, attacker) {
                        logs.extend(log);
                    }

                    Ok(logs)
                } else if !self.stack.is_empty() {
                    Ok(vec![])
                } else {
                    Ok(vec![OpcodeList::new(vec![
                        Opcode::ResetBattleState,
                        Opcode::ChangePhase { phase: Phase::End },
                    ])])
                }
            }
            Phase::End => {
                if let Some(Action::SelectCard { card }) = action {
                    let card = player_in_turn.hand.get(card).unwrap();
                    return Ok(vec![OpcodeList::new(vec![Opcode::MoveCard {
                        card: card.id(),
                        from: Zone::new(self.state.players.player_in_turn()?.id, ZoneKind::Hand),
                        to: Zone::new(self.state.players.player_in_turn()?.id, ZoneKind::Graveyard),
                        reason: MoveReason::Discarded,
                    }])]);
                } else if player_in_turn.hand.len() > self.state.regulation.max_hand_size as usize {
                    return Ok(vec![]);
                }
                let active_player = self
                    .state
                    .players
                    .next_id(self.state.players.player_in_turn()?.id)?;
                let turn = self.state.turn + 1;
                Ok(filter_vec![
                    Some(OpcodeList::new(vec![Opcode::LoadCard {
                        player: self.state.players.player_in_turn()?.id,
                    }])),
                    Some(OpcodeList::new(vec![Opcode::ChangeTurn {
                        turn,
                        player: active_player,
                        phase: Phase::Standby,
                    }])),
                ])
            }
        }
    }
}
