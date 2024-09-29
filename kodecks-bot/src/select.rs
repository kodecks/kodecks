use super::BotContext;
use crate::{
    score::{get_score, ComputedScore},
    Bot, SimpleBot,
};
use kodecks::{action::Action, id::ObjectId, phase::Phase};
use std::sync::Arc;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

pub fn find_select_combination(
    ctx: BotContext,
    cards: Vec<ObjectId>,
) -> Vec<(ObjectId, ComputedScore)> {
    let nop_score = evaluate_select(ctx.clone(), None);

    #[cfg(feature = "rayon")]
    let scored_combinations = cards.into_par_iter();
    #[cfg(not(feature = "rayon"))]
    let scored_combinations = cards.into_iter();

    scored_combinations
        .map(|card| {
            (
                card,
                evaluate_select(ctx.clone(), Some(Action::SelectCard { card })),
            )
        })
        .filter(|(_, score)| *score >= nop_score)
        .collect()
}

fn evaluate_select(mut ctx: BotContext, action: Option<Action>) -> ComputedScore {
    let initial_score = get_score(&ctx.env, ctx.player);

    let env = Arc::make_mut(&mut ctx.env);
    let mut next_action = action;
    let mut player = ctx.player;

    if next_action.is_none() {
        if let Some(actions) = env.last_available_actions() {
            next_action = actions.actions.default_action();
            player = actions.player;
        }
    }

    let initial_turn = env.state.turn;
    while !(matches!(env.state.phase, Phase::End) && env.state.turn > initial_turn) {
        let report = env.process(player, next_action.take());
        if report.endgame.is_ended() {
            break;
        }
        if let Some(available_actions) = &report.available_actions {
            player = available_actions.player;
            next_action = SimpleBot.compute_best_action(Arc::new(env.clone()), available_actions);
        }
    }

    ComputedScore {
        base: initial_score,
        action: get_score(env, ctx.player),
    }
}
