use crate::{
    action::{Action, PlayerAvailableActions},
    card::{Card, Catalog},
    env::{Environment, GameCondition},
    id::ObjectIdCounter,
    log::LogAction,
    player::{PlayerId, PlayerState},
    profile::GameProfile,
    scenario::Scenario,
    sequence::CardSequence,
};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::sync::Arc;

pub struct Game {
    env: Arc<Environment>,
    scenario: Option<Box<dyn Scenario>>,
}

impl Game {
    pub fn new(profile: GameProfile, catalog: &'static Catalog) -> Game {
        let mut rng: SmallRng = profile
            .config
            .rng_seed
            .map(SmallRng::seed_from_u64)
            .unwrap_or_else(SmallRng::from_entropy);

        let mut counter = ObjectIdCounter::default();
        let mut players = profile
            .players
            .into_iter()
            .map(|player| {
                let mut state = PlayerState::new(player.id);
                for item in &player.deck.cards {
                    let archetype = &catalog[item.archetype_id];
                    let card = Card::new(&mut counter, item, archetype, player.id);
                    state.deck.add_top(card);
                }
                if !profile.config.no_deck_shuffle {
                    state.deck.shuffle(&mut counter, &mut rng);
                }
                state
            })
            .collect::<Vec<_>>();

        if !profile.config.no_player_shuffle {
            players.shuffle(&mut rng);
        }

        Game {
            env: Arc::new(Environment::new(profile.config, players)),
            scenario: None,
        }
    }

    pub fn tick(&mut self, player: PlayerId, action: Option<Action>) -> Report {
        let mut report = Arc::make_mut(&mut self.env).process(player, action);
        if let Some(scenario) = &mut self.scenario {
            report.available_actions =
                scenario.override_actions(&self.env, report.available_actions);
        }
        report
    }

    pub fn env(&self) -> &Arc<Environment> {
        &self.env
    }
}

#[derive(Debug, Clone)]
pub struct Report {
    pub available_actions: Option<PlayerAvailableActions>,
    pub logs: Vec<LogAction>,
    pub condition: GameCondition,
}
