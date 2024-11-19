use kodecks::{
    env::{EndgameState, Environment},
    score::Score,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct ComputedScore {
    pub base: i32,
    pub action: i32,
}

impl Score for ComputedScore {
    type Output = i32;

    fn score(&self) -> i32 {
        self.action - self.base
    }
}

impl PartialEq for ComputedScore {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
    }
}

impl Eq for ComputedScore {}

impl PartialOrd for ComputedScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ComputedScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score().cmp(&other.score())
    }
}

pub fn get_score(env: &Environment, side: u8) -> i32 {
    let state = &env.state;
    let player = if let Ok(player) = state.players().get(side) {
        player
    } else {
        return 0;
    };
    let opponent = if let Ok(opponent) = state.players().next_player(side) {
        opponent
    } else {
        return 0;
    };
    let mut score = 0i32;

    score += player.stats.life as i32 / 100;
    score -= opponent.stats.life as i32 / 100 * 2;

    score += player.shards.len() as i32;
    score -= opponent.shards.len() as i32;

    score += player
        .hand
        .items()
        .map(|card| card.score().score())
        .sum::<i32>()
        / 2;
    score += player
        .field
        .items()
        .map(|card| card.score().score())
        .sum::<i32>();
    score += player.abilities.score();

    score -= opponent
        .hand
        .items()
        .map(|card| card.score().score())
        .sum::<i32>()
        / 2;
    score -= opponent
        .field
        .items()
        .map(|card| card.score().score())
        .sum::<i32>();
    score -= opponent.abilities.score();

    if (player.stats.life as f32) < (state.regulation.initial_life as f32 * 0.2) {
        score -= 100;
    }

    score += match env.game_condition() {
        EndgameState::Finished {
            winner: Some(player),
            ..
        } => {
            if player == side {
                1000
            } else {
                -1000
            }
        }
        EndgameState::Finished { winner: None, .. } => -500,
        _ => 0,
    };

    score
}
