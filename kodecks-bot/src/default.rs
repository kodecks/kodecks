use crate::{
    battle, cast, score::ComputedScore, select::find_select_combination, Bot, BotContext, BotFlags,
};
use kodecks::{
    action::{Action, PlayerAvailableActions},
    env::Environment,
    score::Score,
};
use rand::{rngs::SmallRng, SeedableRng};
use std::sync::Arc;
use tracing::*;

#[derive(Debug, Clone)]
pub struct DefaultBot {
    rng: SmallRng,
    flags: BotFlags,
}

pub struct BotBuilder {
    inner: DefaultBot,
}

impl BotBuilder {
    pub fn with_flags(mut self, flags: BotFlags) -> Self {
        self.inner.flags = flags;
        self
    }

    pub fn with_rng(mut self, rng: SmallRng) -> Self {
        self.inner.rng = rng;
        self
    }

    pub fn build(self) -> DefaultBot {
        self.inner
    }
}

impl DefaultBot {
    pub fn builder() -> BotBuilder {
        BotBuilder {
            inner: DefaultBot {
                rng: SmallRng::from_entropy(),
                flags: Default::default(),
            },
        }
    }

    pub fn flags(&self) -> BotFlags {
        self.flags
    }
}

impl Bot for DefaultBot {
    fn compute(
        &mut self,
        env: Arc<Environment>,
        actions: &PlayerAvailableActions,
    ) -> Vec<(Action, ComputedScore)> {
        let ctx: BotContext = BotContext {
            rng: self.rng.clone(),
            player: actions.player,
            env: env.clone(),
            flags: self.flags,
        };

        let selectable_card = actions.actions.selectable_cards();
        let select = find_select_combination(ctx.clone(), selectable_card);
        for (card, score) in &select {
            let name = env
                .state
                .find_card(*card)
                .ok()
                .map(|card| card.archetype().name.clone())
                .unwrap_or_default();
            debug!("Select: {} score: {:?}", name, score);
        }
        let select = select
            .into_iter()
            .map(|(card, score)| (Action::SelectCard { card }, score))
            .max_by_key(|(_, score)| *score);

        let cast_candidates = actions.actions.castable_cards();
        let cast = cast::find_cast_combination(ctx.clone(), cast_candidates);
        for (card, score) in &cast {
            let name = env
                .state
                .find_card(*card)
                .ok()
                .map(|card| card.archetype().name.clone())
                .unwrap_or_default();
            debug!("Cast: {} score: {:?}", name, score);
        }
        let cast = cast
            .into_iter()
            .map(|(card, score)| (Action::CastCard { card }, score))
            .filter(|(_, score)| score.score() > 0)
            .max_by_key(|(_, score)| *score);

        let battle = battle::find_attacker_combination(ctx.clone(), &actions.actions.attackers());
        for (attackers, score) in &battle {
            let attackers = attackers
                .iter()
                .filter_map(|id| env.state.find_card(*id).ok())
                .map(|card| card.archetype().name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            debug!("Battle: {} score: {:?}", attackers, score);
        }

        let battle = battle
            .into_iter()
            .map(|(attackers, score)| {
                (
                    Action::Attack {
                        attackers: attackers.clone(),
                    },
                    score,
                )
            })
            .filter(|(_, score)| score.score() > 0)
            .max_by_key(|(_, score)| *score);

        let opponent = env.state.players().next_player(actions.player);
        let attackers = opponent
            .field
            .attacking_cards()
            .map(|card| card.id())
            .collect::<Vec<_>>();
        let block =
            battle::find_blocker_combination(ctx.clone(), &attackers, &actions.actions.blockers());
        for (pairs, score) in &block {
            let attackers = pairs
                .iter()
                .map(|(attacker, blocker)| {
                    format!(
                        "{} -> {}",
                        env.state
                            .find_card(*attacker)
                            .ok()
                            .map(|card| card.archetype().name.clone())
                            .unwrap_or_default(),
                        env.state
                            .find_card(*blocker)
                            .ok()
                            .map(|card| card.archetype().name.clone())
                            .unwrap()
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            debug!("Block: {} score: {:?}", attackers, score);
        }
        let block = block
            .into_iter()
            .map(|(pairs, score)| (Action::Block { pairs }, score))
            .max_by_key(|(_, score)| *score);

        actions
            .actions
            .default_action(&env)
            .map(|action| {
                (
                    action,
                    ComputedScore {
                        base: 0,
                        action: -100,
                    },
                )
            })
            .into_iter()
            .chain(select)
            .chain(cast)
            .chain(block)
            .chain(battle)
            .collect()
    }
}
