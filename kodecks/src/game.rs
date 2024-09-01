use crate::{
    action::{Action, PlayerAvailableActions},
    card::Catalog,
    env::{Environment, GameCondition},
    log::LogAction,
    player::PlayerId,
    profile::GameProfile,
    scenario::Scenario,
};
use std::sync::Arc;

pub struct Game {
    env: Arc<Environment>,
    scenario: Option<Box<dyn Scenario>>,
}

impl Game {
    pub fn new(profile: GameProfile, catalog: &'static Catalog) -> Game {
        Game {
            env: Arc::new(Environment::new(profile, catalog)),
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
