use super::{Environment, GameState};
use crate::{
    ability::PlayerAbility,
    card::Card,
    condition,
    effect::{EffectActivateContext, EffectTriggerContext},
    error::Error,
    field::FieldState,
    log::LogAction,
    opcode::Opcode,
    player::{PlayerCondition, PlayerZone},
    prelude::{ComputedAttribute, ContinuousEffect, ContinuousItem},
    sequence::CardSequence,
    zone::{CardZone, MoveReason, Zone},
};
use tracing::error;

impl Environment {
    pub fn execute(&mut self, opcode: Opcode) -> Result<Vec<LogAction>, Error> {
        self.timestamp += 1;
        match opcode {
            Opcode::StartGame => Ok(vec![LogAction::GameStarted]),
            Opcode::ChangeTurn {
                turn,
                player,
                phase,
            } => {
                self.state.turn = turn;
                self.state.players.set_player_in_turn(player);
                self.state.phase = phase.clone();
                self.state.players.iter_mut().for_each(|player| {
                    player.reset_counters();
                });
                Ok(vec![LogAction::TurnChanged {
                    turn,
                    player,
                    phase: phase.clone(),
                }])
            }
            Opcode::ChangePhase { phase } => {
                self.state.phase = phase.clone();
                Ok(vec![LogAction::PhaseChanged {
                    phase: phase.clone(),
                }])
            }
            Opcode::SetLife { player, life } => {
                self.state.players.get_mut(player).stats.life = life;
                Ok(vec![LogAction::LifeChanged { player, life }])
            }
            Opcode::ReduceCost { player } => {
                self.state
                    .players
                    .get_mut(player)
                    .hand
                    .items_mut()
                    .for_each(|item| {
                        item.cost_delta = item.cost_delta.saturating_sub(1);
                    });
                Ok(vec![])
            }
            Opcode::GenerateShards {
                player,
                source,
                color,
                amount,
            } => {
                let abilities = &self.state.players.get(player).abilities;
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
                let amount = ((amount as i32) + propagate).max(0) as u32;
                let player = self.state.players.get_mut(player);
                player.shards.add(color, amount);
                Ok(vec![LogAction::ShardsGenerated {
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
                let player = self.state.players.get_mut(player);
                player.shards.consume(color, amount)?;
                Ok(vec![LogAction::ShardsConsumed {
                    player: player.id,
                    source,
                    color,
                    amount,
                }])
            }
            Opcode::BreakShield { card } => {
                let card = self.state.find_card_mut(card)?;
                self.continuous.add(ContinuousItem::new(
                    card,
                    ShieldBroken,
                    condition::OnField(card.id()),
                ));
                Ok(vec![LogAction::ShieldBroken { card: card.id() }])
            }
            Opcode::GenerateCardToken { card } => {
                let id = card.id();
                let player = self.state.players.get_mut(card.controller());
                player.field.push(card);
                Ok(vec![LogAction::CardTokenGenerated { card: id }])
            }
            Opcode::DrawCard { player } => {
                let player = self.state.players.get_mut(player);
                if let Some(mut card) = player.deck.remove_top() {
                    let id = card.id();
                    let from = *card.zone();
                    let to = PlayerZone::new(player.id, Zone::Hand);
                    card.set_timestamp(self.timestamp);
                    card.set_zone(to);
                    player.hand.push(card);
                    player.counters.draw += 1;
                    return Ok(vec![LogAction::CardMoved {
                        card: id,
                        from,
                        to,
                        reason: MoveReason::Draw,
                    }]);
                } else {
                    player.condition.get_or_insert(PlayerCondition::Lose);
                }
                Ok(vec![])
            }
            Opcode::CastCard { player, card, cost } => {
                let player = self.state.players.get_mut(player);
                if let Some(mut card) = player.hand.remove(card) {
                    let id = card.id();
                    let from = *card.zone();
                    let to = PlayerZone::new(player.id, Zone::Field);
                    card.set_timestamp(self.timestamp);
                    card.set_zone(to);
                    player.field.push(card);
                    if cost == 0 {
                        player.counters.free_casted += 1;
                    }
                    return Ok(vec![LogAction::CardMoved {
                        card: id,
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
                let player = self.state.players.get_mut(from.player);
                let card = match from.zone {
                    Zone::Deck => player.deck.remove(card),
                    Zone::Hand => player.hand.remove(card),
                    Zone::Field => player.field.remove(card),
                    Zone::Graveyard => player.graveyard.remove(card),
                    _ => None,
                };
                if let Some(mut card) = card {
                    let id = card.id();
                    if card.is_token() && to.zone != Zone::Field {
                        return Ok(vec![LogAction::CardTokenRemoved { card: id }]);
                    }
                    card.set_timestamp(self.timestamp);
                    card.set_zone(to);
                    card.reset_computed();
                    let player = self.state.players.get_mut(to.player);
                    match to.zone {
                        Zone::Deck => player.deck.push(card),
                        Zone::Hand => player.hand.push(card),
                        Zone::Field => player.field.push(card),
                        Zone::Graveyard => player.graveyard.push(card),
                        _ => (),
                    }
                    return Ok(vec![LogAction::CardMoved {
                        card: id,
                        from,
                        to,
                        reason,
                    }]);
                }
                Ok(vec![])
            }
            Opcode::ShuffleDeck { player } => {
                let player = self.state.players.get_mut(player);
                player.deck.shuffle(&mut self.obj_counter, &mut self.rng);
                Ok(vec![LogAction::DeckShuffled { player: player.id }])
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
                    .map(|id| LogAction::EffectTriggered {
                        source: target.id(),
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
                    player.field.set_card_state(card, state);
                }
                Ok(vec![])
            }
            Opcode::SetBattleState { card, state } => {
                for player in self.state.players.iter_mut() {
                    player.field.set_card_battle_state(card, state);
                }
                Ok(vec![])
            }
            Opcode::ResetBattleState => {
                for player in self.state.players.iter_mut() {
                    for item in player.field.items_mut() {
                        if item.battle.is_none() {
                            item.state = FieldState::Active;
                        }
                        item.battle = None;
                    }
                }
                Ok(vec![])
            }
            Opcode::Attack { attacker, target } => {
                Ok(vec![LogAction::Attacked { attacker, target }])
            }
            Opcode::InflictDamage { player, damage } => {
                let player = self.state.players.get_mut(player);
                player.stats.life = player.stats.life.saturating_sub(damage);
                Ok(vec![LogAction::DamageInflicted {
                    player: player.id,
                    damage,
                }])
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ShieldBroken;

impl ContinuousEffect for ShieldBroken {
    fn apply_card(
        &mut self,
        _state: &GameState,
        source: &Card,
        target: &Card,
        computed: &mut ComputedAttribute,
    ) -> anyhow::Result<()> {
        if target.id() == source.id() {
            if let Some(shields) = &mut computed.shields {
                shields.add(-1);
            }
        }
        Ok(())
    }
}
