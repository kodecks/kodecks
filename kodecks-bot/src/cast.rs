use super::BotContext;
use crate::{
    score::{get_score, ComputedScore},
    Bot, SimpleBot,
};
use kodecks::{action::Action, id::TimedObjectId, phase::Phase};
use std::sync::Arc;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

pub fn find_cast_combination(
    ctx: BotContext,
    cards: Vec<TimedObjectId>,
) -> Vec<(TimedObjectId, ComputedScore)> {
    let base_score = evaluate_cast(ctx.clone(), None);

    #[cfg(feature = "rayon")]
    let scored_combinations = cards.into_par_iter();
    #[cfg(not(feature = "rayon"))]
    let scored_combinations = cards.into_iter();

    scored_combinations
        .map(|card| {
            (
                card,
                ComputedScore {
                    base: base_score,
                    action: evaluate_cast(ctx.clone(), Some(Action::CastCard { card })),
                },
            )
        })
        .collect::<Vec<_>>()
}

fn evaluate_cast(mut ctx: BotContext, action: Option<Action>) -> i32 {
    let env = Arc::make_mut(&mut ctx.env);

    let mut next_action = action;
    let mut current_player = ctx.player;

    if next_action.is_none() {
        if let Some(actions) = env.last_available_actions() {
            next_action = actions.actions.default_action(env);
            current_player = actions.player;
        }
    }

    let initial_turn = env.state.turn;
    while !(matches!(env.state.phase, Phase::End) && env.state.turn > initial_turn) {
        let report = env.process(current_player, next_action.take());
        if report.endgame.is_ended() {
            break;
        }
        if let Some(available_actions) = &report.available_actions {
            current_player = available_actions.player;
            if current_player == ctx.player {
                next_action = available_actions.actions.default_action(env);
            } else {
                next_action =
                    SimpleBot.compute_best_action(Arc::new(env.clone()), available_actions);
            }
        }
    }

    get_score(env, ctx.player)
}
