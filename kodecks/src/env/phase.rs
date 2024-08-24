use super::Environment;
use crate::{
    ability::KeywordAbility,
    action::Action,
    config::DebugFlags,
    error::Error,
    event::CardEvent,
    field::{FieldBattleState, FieldCardState},
    filter_vec,
    opcode::{Opcode, OpcodeList},
    phase::Phase,
    player::PlayerZone,
    target::Target,
    zone::{CardZone, MoveReason, Zone},
};
use std::{iter, vec};

impl Environment {
    fn initialize(&self) -> Result<Vec<OpcodeList>, Error> {
        let initial_life = self.state.config.initial_life;

        Ok(filter_vec![
            Some(OpcodeList::new(vec![Opcode::StartGame],)),
            self.state.players.iter().flat_map(|player| {
                filter_vec![Some(OpcodeList::new(vec![Opcode::SetLife {
                    player: player.id,
                    life: initial_life,
                }])),]
            }),
            self.state.players.iter().flat_map(|player| {
                let opcodes = iter::repeat(OpcodeList::new(vec![Opcode::DrawCard {
                    player: player.id,
                }]))
                .take(self.state.config.initial_hand_size as usize);
                filter_vec![opcodes,]
            }),
            Some(OpcodeList::new(vec![Opcode::ChangeTurn {
                turn: 1,
                player: self.state.players.player_in_turn(),
                phase: self.state.phase.clone(),
            },],)),
        ])
    }

    pub fn process_player_phase(
        &self,
        action: Option<Action>,
        phase: &mut Phase,
    ) -> Result<Vec<OpcodeList>, Error> {
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
                    .items()
                    .filter(|item| {
                        item.card.computed().is_hex()
                            && !self
                                .stack
                                .iter()
                                .any(|stack| stack.source == item.card.id())
                    })
                    .map(|item| item.card.id())
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
                        to: PlayerZone::new(card.owner(), Zone::Graveyard),
                        reason: MoveReason::Move,
                    }])
                })
                .collect());
        }

        let player_in_turn = &self.state.players.get(self.state.players.player_in_turn());
        match phase {
            Phase::Standby => {
                let next_phase = Phase::Draw;
                Ok(filter_vec![
                    if self.state.turn == 1 {
                        None
                    } else {
                        Some(OpcodeList::new(vec![Opcode::ReduceCost {
                            player: self.state.players.player_in_turn(),
                        }]))
                    },
                    Some(OpcodeList::new(vec![Opcode::ChangePhase {
                        phase: next_phase
                    }],)),
                ])
            }
            Phase::Draw => {
                if player_in_turn.counters.draw == 0 {
                    Ok(vec![OpcodeList::new(vec![Opcode::DrawCard {
                        player: player_in_turn.id,
                    }])])
                } else {
                    Ok(vec![OpcodeList::new(vec![Opcode::ChangePhase {
                        phase: Phase::Main,
                    }])])
                }
            }
            Phase::Main => {
                let logs = match action {
                    Some(Action::CastCard { card }) => {
                        let item = player_in_turn.hand.get_item(card)?;
                        let color = item.card.computed().color;
                        let amount = if self.state.config.debug.contains(DebugFlags::IGNORE_COST) {
                            0
                        } else {
                            item.card.computed().cost.value() as u32
                        };
                        if player_in_turn.shards.get(color) < amount {
                            return Err(Error::InsufficientShards { color, amount });
                        }
                        if item.card.computed().is_creature()
                            && player_in_turn.counters.cast_creatures > 0
                        {
                            return Err(Error::CreatureAlreadyCasted);
                        }
                        filter_vec![
                            Some(OpcodeList::new(filter_vec![
                                if amount > 0 {
                                    Some(Opcode::ConsumeShards {
                                        player: self.state.players.player_in_turn(),
                                        source: item.card.id(),
                                        color: item.card.computed().color,
                                        amount,
                                    })
                                } else {
                                    None
                                },
                                Some(Opcode::CastCard {
                                    player: self.state.players.player_in_turn(),
                                    card: item.card.id(),
                                }),
                            ],)),
                            self.apply_event(CardEvent::Casted, &item.card, &item.card)
                                .ok()
                                .into_iter()
                                .flatten(),
                            self.apply_event_any(CardEvent::AnyCasted, &item.card)
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
                let active_player = self.state.players.next(self.state.players.player_in_turn());
                let active_player = &self.state.players.get(active_player);

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
                                card: *blocker,
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
                    let item = active_player.hand.get_item(card)?;
                    let color = item.card.computed().color;
                    let amount = if self.state.config.debug.contains(DebugFlags::IGNORE_COST) {
                        0
                    } else {
                        item.card.computed().cost.value() as u32
                    };
                    if active_player.shards.get(color) < amount {
                        return Err(Error::InsufficientShards { color, amount });
                    }
                    if item.card.computed().is_creature()
                        && active_player.counters.cast_creatures > 0
                    {
                        return Err(Error::CreatureAlreadyCasted);
                    }
                    Ok(filter_vec![
                        Some(OpcodeList::new(filter_vec![
                            if amount > 0 {
                                Some(Opcode::ConsumeShards {
                                    player: active_player.id,
                                    source: item.card.id(),
                                    color: item.card.computed().color,
                                    amount,
                                })
                            } else {
                                None
                            },
                            Some(Opcode::CastCard {
                                player: active_player.id,
                                card: item.card.id(),
                            }),
                        ],)),
                        self.apply_event(CardEvent::Casted, &item.card, &item.card)
                            .ok()
                            .into_iter()
                            .flatten(),
                        self.apply_event_any(CardEvent::AnyCasted, &item.card)
                            .ok()
                            .into_iter()
                            .flatten(),
                    ])
                } else {
                    Ok(vec![])
                }
            }
            Phase::Battle => {
                let target_player = self.state.players.next(self.state.players.player_in_turn());
                let target = &self.state.players.get(target_player);
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
                                Target::Player(target_player)
                            },
                        },
                        Opcode::SetBattleState {
                            card: attacker.id(),
                            state: Some(FieldBattleState::Attacked),
                        },
                        Opcode::SetFieldCardState {
                            card: attacker.id(),
                            state: FieldCardState::Exhausted,
                        },
                    ]));

                    if blocker.is_none() && attacker_power > 0 {
                        logs.push(OpcodeList::new(vec![Opcode::InflictDamage {
                            player: target_player,
                            damage: attacker_power,
                        }]));

                        if let Ok(log) = self.apply_event(
                            CardEvent::DealtDamage {
                                player: target_player,
                                amount: attacker_power,
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

                        let blocker_has_toxic = blocker
                            .computed()
                            .abilities
                            .contains(&KeywordAbility::Toxic);
                        let blocker_power = blocker.computed().power.unwrap_or_default().value();
                        if (blocker_power > 0 && attacker_power < blocker_power)
                            || blocker_has_toxic
                        {
                            if let Ok(log) =
                                self.apply_event(CardEvent::Destroyed, attacker, attacker)
                            {
                                logs.extend(log);
                            }
                        }

                        let attacker_has_toxic = attacker
                            .computed()
                            .abilities
                            .contains(&KeywordAbility::Toxic);
                        if (attacker_power > 0 && blocker_power <= attacker_power)
                            || attacker_has_toxic
                        {
                            if let Ok(log) =
                                self.apply_event(CardEvent::Destroyed, blocker, blocker)
                            {
                                logs.extend(log);
                            }
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
                        from: PlayerZone::new(self.state.players.player_in_turn(), Zone::Hand),
                        to: PlayerZone::new(self.state.players.player_in_turn(), Zone::Graveyard),
                        reason: MoveReason::Discarded,
                    }])]);
                }
                let active_player = self.state.players.next(self.state.players.player_in_turn());
                let turn = self.state.turn + 1;
                Ok(vec![OpcodeList::new(vec![Opcode::ChangeTurn {
                    turn,
                    player: active_player,
                    phase: Phase::Standby,
                }])])
            }
        }
    }
}
