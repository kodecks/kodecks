use crate::{score::ComputedScore, Bot};
use kodecks::{
    action::{Action, AvailableAction, PlayerAvailableActions},
    env::Environment,
    id::TimedCardId,
    score::Score,
};
use std::sync::Arc;
use tracing::debug;

pub struct SimpleBot;

impl Bot for SimpleBot {
    fn compute(
        &mut self,
        env: Arc<Environment>,
        actions: &PlayerAvailableActions,
    ) -> Vec<(Action, ComputedScore)> {
        for action in actions.actions.as_ref() {
            if let AvailableAction::SelectCard { cards } = action {
                let best_candidate = cards
                    .iter()
                    .filter_map(|id| env.state.find_card(*id).ok())
                    .map(|card| {
                        (
                            card.timed_id(),
                            ComputedScore {
                                base: 0,
                                action: card.score().score()
                                    * if card.zone().player == actions.player {
                                        1
                                    } else {
                                        -1
                                    },
                            },
                        )
                    })
                    .max_by_key(|(_, score)| score.score());
                if let Some((card, score)) = best_candidate {
                    return vec![(Action::SelectCard { card }, score)];
                }
            }

            if let AvailableAction::Attack { attackers } = action {
                let player = if let Ok(player) = env.state.players().get(actions.player) {
                    player
                } else {
                    return vec![];
                };
                let opponent = if let Ok(opponent) = env.state.players().next_player(actions.player)
                {
                    opponent
                } else {
                    return vec![];
                };

                let blockers = opponent
                    .field
                    .active_cards()
                    .filter_map(|card| card.computed().power)
                    .map(|power| power.value())
                    .collect::<Vec<_>>();
                let blocker_power_sum = blockers.iter().sum::<u32>();
                let max_blocker_power = blockers.iter().copied().max().unwrap_or_default();
                if blocker_power_sum as i32 >= player.stats.life {
                    return vec![(
                        Action::Attack { attackers: vec![] },
                        ComputedScore::default(),
                    )];
                }
                let attackers = attackers
                    .iter()
                    .filter_map(|id| env.state.find_card(*id).ok())
                    .filter(|card| {
                        let power = card
                            .computed()
                            .power
                            .map(|power| power.value())
                            .unwrap_or_default();
                        power > 0 && power > max_blocker_power
                    })
                    .map(|card| card.timed_id())
                    .collect::<Vec<_>>();
                if !attackers.is_empty() {
                    return vec![(Action::Attack { attackers }, ComputedScore::default())];
                }
            }

            if let AvailableAction::Block {
                blockers,
                attackers,
            } = action
            {
                let player = if let Ok(player) = env.state.players().get(actions.player) {
                    player
                } else {
                    return vec![];
                };
                let mut blockers = blockers
                    .iter()
                    .filter_map(|id| env.state.find_card(*id).ok())
                    .collect::<Vec<_>>();
                blockers.sort_by_key(|card| {
                    card.computed()
                        .power
                        .map(|power| power.value())
                        .unwrap_or_default() as i32
                });
                let mut attackers = attackers
                    .iter()
                    .filter_map(|id| env.state.find_card(*id).ok())
                    .collect::<Vec<_>>();
                attackers.sort_by_key(|card| {
                    card.computed()
                        .power
                        .map(|power| power.value())
                        .unwrap_or_default() as i32
                });
                let mut pairs = vec![];
                while !attackers.is_empty() && !blockers.is_empty() {
                    let attackers_power_sum = attackers
                        .iter()
                        .map(|card| {
                            card.computed()
                                .power
                                .map(|power| power.value())
                                .unwrap_or_default()
                        })
                        .sum::<u32>();
                    let attacker = attackers.pop().unwrap();
                    let attacker_power = attacker
                        .computed()
                        .power
                        .map(|power| power.value())
                        .unwrap_or_default();

                    debug!(
                        "attacker_power_sum: {} {}",
                        attackers_power_sum, player.stats.life
                    );

                    if attacker_power > 0 {
                        let blocker = blockers.iter().position(|blocker| {
                            blocker
                                .computed()
                                .power
                                .map(|power| power.value())
                                .unwrap_or_default()
                                > attacker_power
                                || attackers_power_sum as i32 >= player.stats.life
                        });
                        if let Some(index) = blocker {
                            let blocker = blockers.remove(index);
                            pairs.push((attacker.timed_id(), blocker.timed_id()));
                        }
                    }
                }
                return vec![(Action::Block { pairs }, ComputedScore::default())];
            }

            if let AvailableAction::CastCard { cards } = action {
                let best_candidate = cards
                    .iter()
                    .filter_map(|id| env.state.find_card(*id).ok())
                    .map(|card| {
                        (
                            card.timed_id(),
                            ComputedScore {
                                base: 0,
                                action: card.score().score(),
                            },
                        )
                    })
                    .max_by_key(|(_, score)| score.score());
                if let Some((card, score)) = best_candidate {
                    return vec![(Action::CastCard { card }, score)];
                }
            }
        }

        actions
            .actions
            .default_action(&env)
            .map(|action| (action, ComputedScore::default()))
            .into_iter()
            .collect()
    }
}
