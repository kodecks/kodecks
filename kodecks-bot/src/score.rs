use kodecks::{
    env::{GameCondition, GameState},
    player::PlayerId,
    score::Score,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct ComputedScore {
    pub base: i32,
    pub action: i32,
}

impl Score for ComputedScore {
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

pub fn get_score(state: &GameState, side: PlayerId) -> i32 {
    let player = state.players().get(side);
    let opponent = state.players().get(state.players.next(side));
    let mut score = 0i32;

    score += player.stats.life as i32 / 100;
    score -= opponent.stats.life as i32 / 100 * 2;

    score += player.shards.len() as i32;
    score -= opponent.shards.len() as i32;

    score += player
        .hand
        .items()
        .map(|item| item.card.score())
        .sum::<i32>()
        / 2;
    score += player.field.items().map(|item| item.score()).sum::<i32>();
    score += player.abilities.score();

    score -= opponent
        .hand
        .items()
        .map(|item| item.card.score())
        .sum::<i32>()
        / 2;
    score -= opponent.field.items().map(|item| item.score()).sum::<i32>();
    score -= opponent.abilities.score();

    if (player.stats.life as f32) < (state.config.initial_life as f32 * 0.2) {
        score -= 100;
    }

    score += match state.check_game_condition() {
        GameCondition::Win(player) => {
            if player == side {
                1000
            } else {
                -1000
            }
        }
        GameCondition::Draw => -500,
        _ => 0,
    };

    score
}
