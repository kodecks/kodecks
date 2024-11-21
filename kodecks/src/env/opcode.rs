use super::{EndgameReason, Environment};
use crate::{
    ability::PlayerAbility,
    effect::{ContinuousCardEffectContext, EffectActivateContext, EffectTriggerContext},
    error::ActionError,
    field::{FieldBattleState, FieldState},
    log::GameLog,
    opcode::Opcode,
    player::{PlayerEndgameState, Zone},
    prelude::{ContinuousEffect, ContinuousItem},
    sequence::CardSequence,
    target::Target,
    zone::{CardZone, MoveReason, ZoneKind},
};
use tracing::error;

impl Environment {
    pub fn execute(&mut self, opcode: Opcode) -> Result<Vec<GameLog>, ActionError> {
        self.timestamp += 1;
        match opcode {
            Opcode::StartGame => Ok(vec![GameLog::GameStarted]),
            Opcode::ChangeTurn {
                turn,
                player,
                phase,
            } => {
                self.state.turn = turn;
                self.state.players.set_player_in_turn(player);
                self.state.phase = phase;
                self.state.players.iter_mut().for_each(|player| {
                    player.reset_counters();
                });
                Ok(vec![
                    GameLog::TurnChanged { turn, player },
                    GameLog::PhaseChanged { phase },
                ])
            }
            Opcode::ChangePhase { phase } => {
                self.state.phase = phase;
                Ok(vec![GameLog::PhaseChanged { phase }])
            }
            Opcode::SetLife { player, life } => {
                self.state.players.get_mut(player)?.stats.life = life;
                Ok(vec![GameLog::LifeChanged { player, life }])
            }
            Opcode::ReduceCost { player } => {
                self.state
                    .players
                    .get_mut(player)?
                    .hand
                    .items_mut()
                    .for_each(|card| {
                        card.set_hand_cost_delta(card.hand_cost_delta().saturating_sub(1));
                    });
                Ok(vec![])
            }
            Opcode::GenerateShards {
                player,
                source,
                color,
                amount,
            } => {
                let abilities = &self.state.players.get(player)?.abilities;
                let propagate = abilities
                    .iter()
                    .find_map(|a| {
                        if let PlayerAbility::Propagate(n) = a {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                let source = self.state.find_card(source)?.snapshot();
                let amount = ((amount as i32) + propagate).max(0) as u8;
                let player = self.state.players.get_mut(player)?;
                player.shards.add(color, amount);
                Ok(vec![GameLog::ShardsEarned {
                    player: player.id,
                    source,
                    color,
                    amount,
                }])
            }
            Opcode::ConsumeShards {
                player,
                source,
                color,
                amount,
            } => {
                let source = self.state.find_card(source)?.snapshot();
                let player = self.state.players.get_mut(player)?;
                player.shards.consume(color, amount)?;
                Ok(vec![GameLog::ShardsSpent {
                    player: player.id,
                    source,
                    color,
                    amount,
                }])
            }
            Opcode::BreakShield { card } => {
                let card = self.state.find_card(card)?;
                self.continuous.add(ContinuousItem::new(
                    card,
                    ShieldBroken,
                    Target::Card(card.id()),
                ));
                Ok(vec![GameLog::ShieldBroken {
                    card: card.snapshot(),
                }])
            }
            Opcode::GenerateCardToken { card } => {
                let snapshot = card.snapshot();
                let player = self.state.players.get_mut(card.controller())?;
                player.field.push(card);
                Ok(vec![GameLog::CardTokenGenerated { card: snapshot }])
            }
            Opcode::DrawCard { player } => {
                let player = self.state.players.get_mut(player)?;
                if let Some(mut card) = player.deck.remove_top() {
                    let from = *card.zone();
                    let to = Zone::new(player.id, ZoneKind::Hand);
                    let controller = card.controller();
                    card.set_zone(to);
                    let snapshot = card.snapshot();
                    player.hand.push(card);
                    player.counters.draw += 1;
                    return Ok(vec![GameLog::CardMoved {
                        player: controller,
                        card: snapshot,
                        from,
                        to,
                        reason: MoveReason::Draw,
                    }]);
                } else {
                    player
                        .endgame
                        .get_or_insert(PlayerEndgameState::Lose(EndgameReason::DeckOut));
                }
                Ok(vec![])
            }
            Opcode::CastCard { player, card, cost } => {
                let player = self.state.players.get_mut(player)?;
                if let Some(mut card) = player.hand.remove(card) {
                    let from = *card.zone();
                    let to = Zone::new(player.id, ZoneKind::Field);
                    let controller = card.controller();
                    card.set_zone(to);
                    let snapshot = card.snapshot();
                    player.field.push(card);
                    if cost == 0 {
                        player.counters.free_casted += 1;
                    }
                    return Ok(vec![GameLog::CardMoved {
                        player: controller,
                        card: snapshot,
                        from,
                        to,
                        reason: MoveReason::Casted,
                    }]);
                }
                Ok(vec![])
            }
            Opcode::MoveCard {
                card,
                from,
                to,
                reason,
            } => {
                let player = self.state.players.get_mut(from.player)?;
                let card = match from.kind {
                    ZoneKind::Deck => player.deck.remove(card),
                    ZoneKind::Hand => player.hand.remove(card),
                    ZoneKind::Field => player.field.remove(card),
                    ZoneKind::Graveyard => player.graveyard.remove(card),
                };
                if let Some(mut card) = card {
                    if card.is_token() && to.kind != ZoneKind::Field {
                        let owner = self.state.players.get_mut(card.owner())?;
                        card.set_zone(Zone {
                            player: owner.id,
                            ..*card.zone()
                        });
                        let snapshot = card.snapshot();
                        owner.limbo.push(card);
                        return Ok(vec![GameLog::CardTokenDestroyed { card: snapshot }]);
                    }
                    let controller = card.controller();
                    card.set_zone(to);
                    let snapshot = card.snapshot();
                    let player = self.state.players.get_mut(to.player)?;
                    match to.kind {
                        ZoneKind::Deck => player.deck.push(card),
                        ZoneKind::Hand => player.hand.push(card),
                        ZoneKind::Field => player.field.push(card),
                        ZoneKind::Graveyard => player.graveyard.push(card),
                    }
                    return Ok(vec![GameLog::CardMoved {
                        player: controller,
                        card: snapshot,
                        from,
                        to,
                        reason,
                    }]);
                }
                Ok(vec![])
            }
            Opcode::ShuffleDeck { player } => {
                let player = self.state.players.get_mut(player)?;
                player.deck.shuffle(&mut self.obj_counter, &mut self.rng);
                Ok(vec![GameLog::DeckShuffled { player: player.id }])
            }
            Opcode::TriggerEvent {
                source,
                target,
                event,
            } => {
                let source = self.state.find_card(source)?;
                let target = self.state.find_card(target)?;
                let mut ctx = EffectActivateContext::new(&self.state, source, target);

                if let Err(err) = target.effect().activate(event, &mut ctx) {
                    error!("Error triggering effect: {:?}", err);
                };

                let (continuous, stack) = ctx.into_inner();
                let log = stack
                    .iter()
                    .map(|id| GameLog::EffectActivated {
                        source: target.snapshot(),
                        id: *id,
                    })
                    .collect::<Vec<_>>();

                let mut ctx = EffectTriggerContext::new(&self.state, &mut self.obj_counter, target);
                let mut effect = target.effect();
                for id in stack.into_iter().chain(continuous) {
                    if let Err(err) = effect.trigger(id, &mut ctx) {
                        error!("Error triggering effect: {:?}", err);
                    }
                }
                let (continuous, stack) = ctx.into_inner();
                self.continuous.extend(continuous);
                self.stack.extend(stack);
                self.state.find_card_mut(target.id())?.set_effect(effect);

                Ok(log)
            }
            Opcode::SetFieldState { card, state } => {
                for player in self.state.players.iter_mut() {
                    player.field.set_card_field_state(card, state);
                }
                Ok(vec![])
            }
            Opcode::SetBattleState { card, state } => {
                let attacker = self.state.find_card(card)?.snapshot();
                let mut logs = Vec::new();
                for player in self.state.players.iter_mut() {
                    if player.field.set_card_battle_state(card, state)
                        && state == Some(FieldBattleState::Attacking)
                    {
                        logs.push(GameLog::AttackDeclared {
                            attacker: attacker.clone(),
                        });
                    }
                }
                Ok(logs)
            }
            Opcode::ResetBattleState => {
                for player in self.state.players.iter_mut() {
                    for card in player.field.items_mut() {
                        if card.battle_state().is_none() {
                            card.set_field_state(FieldState::Active);
                        }
                        card.set_battle_state(None);
                    }
                }
                Ok(vec![])
            }
            Opcode::Attack { attacker, target } => {
                let attacker = self.state.find_card(attacker)?;
                Ok(vec![match target {
                    Target::Card(target) => {
                        let blocker = self.state.find_card(target)?;
                        GameLog::CreatureAttackedCreature {
                            attacker: attacker.snapshot(),
                            blocker: blocker.snapshot(),
                        }
                    }
                    Target::Player(target) => GameLog::CreatureAttackedPlayer {
                        attacker: attacker.snapshot(),
                        player: target,
                    },
                }])
            }
            Opcode::InflictDamage { player, amount } => {
                let player = self.state.players.get_mut(player)?;
                if let Ok(amount) = amount.try_into() {
                    player.stats.life = player.stats.life.saturating_sub(amount);
                }
                Ok(vec![
                    GameLog::DamageTaken {
                        player: player.id,
                        amount,
                    },
                    GameLog::LifeChanged {
                        player: player.id,
                        life: player.stats.life,
                    },
                ])
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ShieldBroken;

impl ContinuousEffect for ShieldBroken {
    fn apply_card(&mut self, ctx: &mut ContinuousCardEffectContext) -> anyhow::Result<bool> {
        if ctx.target.id() == ctx.source.id() {
            if let Some(shields) = &mut ctx.computed.shields {
                shields.add(-1);
            }
        }
        Ok(true)
    }
}
