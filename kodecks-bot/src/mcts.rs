use crate::{score::ComputedScore, Bot};
use kodecks::{
    action::{Action, AvailableAction, PlayerAvailableActions},
    env::{EndgameState, Environment},
    id::{TimedCardId, TimedObjectId},
};
use rand::seq::{IteratorRandom, SliceRandom};
use std::sync::Arc;

#[derive(Default)]
pub struct MctsBot {}

impl Bot for MctsBot {
    fn compute(
        &mut self,
        env: Arc<Environment>,
        actions: &PlayerAvailableActions,
    ) -> Vec<(Action, ComputedScore)> {
        let mut mcts = Mcts::new(&env, actions.player);
        if let Some(action) = mcts.search(200, 1.4) {
            return vec![(action, ComputedScore::default())];
        }
        vec![]
    }
}

#[derive(Clone)]
pub struct State {
    env: Environment,
    available_actions: Vec<(u8, Action)>,
}

impl State {
    pub fn new(env: &Environment) -> Self {
        let available_actions = Self::get_available_actions(env.last_available_actions(), env);
        Self {
            env: env.clone(),
            available_actions,
        }
    }

    fn get_available_actions(
        actions: Option<&PlayerAvailableActions>,
        env: &Environment,
    ) -> Vec<(u8, Action)> {
        if let Some(actions) = actions {
            let player = actions.player;
            actions
                .actions
                .iter()
                .flat_map(|actions| match actions {
                    AvailableAction::SelectCard { cards } => cards
                        .iter()
                        .map(|&card| (player, Action::SelectCard { card }))
                        .collect(),
                    AvailableAction::FetchCard { cards } => cards
                        .iter()
                        .map(|&card| (player, Action::FetchCard { card }))
                        .collect(),
                    AvailableAction::CastCard { cards } => cards
                        .iter()
                        .map(|&card| (player, Action::CastCard { card }))
                        .collect(),
                    AvailableAction::Attack { attackers } => possible_combinations(attackers)
                        .into_iter()
                        .map(|attackers| (player, Action::Attack { attackers }))
                        .collect(),
                    AvailableAction::Block {
                        attackers,
                        blockers,
                    } => possible_combinations(blockers)
                        .into_iter()
                        .map(|blockers| {
                            (
                                player,
                                Action::Block {
                                    pairs: select_blocking_pair(attackers, &blockers, env),
                                },
                            )
                        })
                        .chain(Some((player, Action::Block { pairs: vec![] })))
                        .collect(),
                    AvailableAction::EndTurn => vec![(player, Action::EndTurn)],
                    _ => vec![],
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn progress(&self, player: u8, action: Action) -> Self {
        let mut new_state = Self {
            env: self.env.clone(),
            available_actions: vec![],
        };
        let mut action = Some(action);
        while !new_state.env.game_condition().is_ended() {
            let report = new_state.env.process(player, action.take());
            new_state.available_actions =
                Self::get_available_actions(report.available_actions.as_ref(), &new_state.env);
            if !new_state.available_actions.is_empty() {
                break;
            }
        }
        new_state
    }
}

struct Node {
    state: State,
    action: Option<Action>,
    parent: Option<*mut Self>,
    children: Vec<Self>,
    visits: u32,
    wins: f64,
}

impl Node {
    fn new(state: State, parent: Option<*mut Self>) -> Self {
        Self {
            state,
            action: None,
            parent,
            children: vec![],
            visits: 0,
            wins: 0.0,
        }
    }

    fn expand(&mut self, possible_actions: Vec<(u8, Action)>) {
        for (player, action) in possible_actions {
            let child = Node {
                state: self.state.progress(player, action.clone()),
                action: Some(action),
                parent: Some(self),
                children: Vec::new(),
                visits: 0,
                wins: 0.0,
            };
            self.children.push(child);
        }
    }

    fn ucb1(&self, parent_visits: u32, exploration_constant: f64) -> f64 {
        if self.visits == 0 {
            f64::INFINITY
        } else {
            let exploit = self.wins / self.visits as f64;
            let explore = (2.0 * (parent_visits as f64).ln() / self.visits as f64).sqrt();
            exploit + exploration_constant * explore
        }
    }

    fn select_child(&self, exploration_constant: f64) -> Option<&Self> {
        let unvisited = self
            .children
            .iter()
            .filter(|child| child.visits == 0)
            .collect::<Vec<_>>();

        if !unvisited.is_empty() {
            return unvisited.choose(&mut rand::thread_rng()).copied();
        }

        self.children.iter().max_by(|a, b| {
            let a_ucb = a.ucb1(self.visits, exploration_constant);
            let b_ucb = b.ucb1(self.visits, exploration_constant);
            a_ucb.partial_cmp(&b_ucb).unwrap()
        })
    }

    fn backpropagate(&mut self, result: f64) {
        let mut current = Some(self);
        while let Some(curr_node) = current {
            curr_node.visits += 1;
            curr_node.wins += result;
            current = unsafe { curr_node.parent.map(|p| &mut *p) };
        }
    }

    fn simulate(&self, player: u8) -> f64 {
        let mut env = self.state.env.clone();
        while !env.game_condition().is_ended() {
            let available_actions =
                State::get_available_actions(env.last_available_actions(), &env);
            let action = available_actions.choose(&mut rand::thread_rng());
            if let Some((player, action)) = action {
                env.process(*player, Some(action.clone()));
            } else {
                env.process(0, None);
            }
        }
        match env.game_condition() {
            EndgameState::Finished { winner, .. } => match winner {
                Some(winner) => {
                    if winner == player {
                        1.0
                    } else {
                        0.0
                    }
                }
                None => 0.5,
            },
            _ => 0.0,
        }
    }
}

struct Mcts {
    root: Node,
    player: u8,
}

impl Mcts {
    fn new(env: &Environment, player: u8) -> Self {
        Self {
            root: Node::new(State::new(env), None),
            player,
        }
    }

    fn search(&mut self, num_iterations: u32, exploration_constant: f64) -> Option<Action> {
        for _ in 0..num_iterations {
            let mut current_node = &mut self.root;
            while !current_node.children.is_empty() {
                let selected = current_node.select_child(exploration_constant)? as *const Node;
                current_node = current_node
                    .children
                    .iter_mut()
                    .find(|c| *c as *const Node == selected)?;

                if current_node.visits == 0 {
                    let simulation_result = current_node.simulate(self.player);
                    current_node.backpropagate(simulation_result);
                    continue;
                }
            }

            let possible_actions = current_node.state.available_actions.clone();
            if !possible_actions.is_empty() && current_node.visits > 0 {
                current_node.expand(possible_actions);

                if let Some(idx) = (0..current_node.children.len()).choose(&mut rand::thread_rng())
                {
                    let child = &mut current_node.children[idx];
                    let simulation_result = child.simulate(self.player);
                    child.backpropagate(simulation_result);
                }
            } else if current_node.visits == 0 {
                let simulation_result = current_node.simulate(self.player);
                current_node.backpropagate(simulation_result);
            }
        }

        let bast_node = self.root.children.iter().max_by_key(|child| child.visits);
        if let Some(best_node) = bast_node {
            return best_node.action.clone();
        }
        None
    }
}

fn possible_combinations(candidates: &[TimedObjectId]) -> Vec<Vec<TimedObjectId>> {
    let mut combinations = vec![];
    for k in 0..=candidates.len() {
        let mut result = Vec::new();
        let mut current = Vec::new();
        backtrack(candidates, k, 0, &mut current, &mut result);
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

fn select_blocking_pair(
    attackers: &[TimedObjectId],
    blockers: &[TimedObjectId],
    env: &Environment,
) -> Vec<(TimedObjectId, TimedObjectId)> {
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
    attackers
        .into_iter()
        .zip(blockers)
        .map(|(a, b)| (a.timed_id(), b.timed_id()))
        .collect()
}
