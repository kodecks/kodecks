use super::BotContext;
use crate::{
    score::{get_score, ComputedScore},
    Bot, SimpleBot,
};
use kodecks::{action::Action, env::Environment, id::ObjectId, phase::Phase};
use std::sync::Arc;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

pub fn find_attacker_combination(
    ctx: BotContext,
    attackers: &[ObjectId],
) -> Vec<(Vec<ObjectId>, ComputedScore)> {
    if attackers.is_empty() {
        return vec![];
    }

    let base_score = evaluate_battle(&ctx.env, ctx.player, None);

    let combinations = possible_attacker_combinations(attackers);
    #[cfg(feature = "rayon")]
    let scored_combinations = combinations.into_par_iter();
    #[cfg(not(feature = "rayon"))]
    let scored_combinations = combinations.into_iter();

    scored_combinations
        .map(|attackers| {
            (
                attackers.clone(),
                ComputedScore {
                    base: base_score,
                    action: evaluate_battle(
                        &ctx.env,
                        ctx.player,
                        Some(Action::Attack { attackers }),
                    ),
                },
            )
        })
        .collect()
}

fn possible_attacker_combinations(attackers: &[ObjectId]) -> Vec<Vec<ObjectId>> {
    let mut combinations = vec![];
    for k in 0..=attackers.len() {
        let mut result = Vec::new();
        let mut current = Vec::new();
        backtrack(attackers, k, 0, &mut current, &mut result);
        combinations.extend(result);
    }
    combinations
}

fn backtrack<T: Clone>(
    arr: &[T],
    k: usize,
    start: usize,
    current: &mut Vec<T>,
    result: &mut Vec<Vec<T>>,
) {
    if current.len() == k {
        result.push(current.clone());
        return;
    }

    for i in start..arr.len() {
        current.push(arr[i].clone());
        backtrack(arr, k, i + 1, current, result);
        current.pop();
    }
}

pub fn find_blocker_combination(
    ctx: BotContext,
    attackers: &[ObjectId],
    blockers: &[ObjectId],
) -> Vec<(Vec<(ObjectId, ObjectId)>, ComputedScore)> {
    if attackers.is_empty() || blockers.is_empty() {
        return vec![];
    }

    let base_score = evaluate_battle(&ctx.env, ctx.player, None);

    let combinations = possible_battle_combinations(attackers, blockers);

    #[cfg(feature = "rayon")]
    let comb = combinations.into_par_iter();
    #[cfg(not(feature = "rayon"))]
    let comb = combinations.into_iter();

    comb.map(|pairs| {
        (
            pairs.clone(),
            ComputedScore {
                base: base_score,
                action: evaluate_battle(&ctx.env, ctx.player, Some(Action::Block { pairs })),
            },
        )
    })
    .collect()
}

fn possible_battle_combinations(
    attackers: &[ObjectId],
    blockers: &[ObjectId],
) -> Vec<Vec<(ObjectId, ObjectId)>> {
    let mut pairs = vec![];
    for attacker in attackers {
        for blocker in blockers {
            pairs.push((*attacker, *blocker));
        }
    }

    let mut combinations = vec![];
    for k in 0..=pairs.len() {
        let mut result = Vec::new();
        let mut current = Vec::new();
        backtrack_pair(&pairs, k, 0, &mut current, &mut result);
        combinations.extend(result);
    }

    combinations
}

fn backtrack_pair(
    arr: &[(ObjectId, ObjectId)],
    k: usize,
    start: usize,
    current: &mut Vec<(ObjectId, ObjectId)>,
    result: &mut Vec<Vec<(ObjectId, ObjectId)>>,
) {
    if current.len() == k {
        result.push(current.clone());
        return;
    }

    for i in start..arr.len() {
        let (a, b) = arr[i];
        if current.iter().all(|&(x, y)| x != a && y != b) {
            current.push(arr[i]);
            backtrack_pair(arr, k, i + 1, current, result);
            current.pop();
        }
    }
}

fn evaluate_battle(env: &Environment, player: u8, action: Option<Action>) -> i32 {
    let mut next_action = action;
    let mut current_player = player;

    if next_action.is_none() {
        if let Some(actions) = env.last_available_actions() {
            next_action = actions.actions.default_action();
            current_player = actions.player;
        }
    }

    let initial_turn = env.state.turn;
    let mut env = env.clone();
    while !(matches!(env.state.phase, Phase::End) && env.state.turn > initial_turn) {
        let report = env.process(current_player, next_action.take());
        if report.endgame.is_ended() {
            break;
        }
        if let Some(available_actions) = &report.available_actions {
            current_player = available_actions.player;
            if current_player == player && !matches!(env.state.phase, Phase::Block) {
                next_action = available_actions.actions.default_action();
            } else {
                next_action =
                    SimpleBot.compute_best_action(Arc::new(env.clone()), available_actions);
            }
        }
    }

    get_score(&env, player)
}
