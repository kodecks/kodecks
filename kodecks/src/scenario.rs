use crate::{action::PlayerAvailableActions, env::Environment};

pub trait Scenario: 'static + Send + Sync {
    fn override_actions(
        &mut self,
        env: &Environment,
        actions: Option<PlayerAvailableActions>,
    ) -> Option<PlayerAvailableActions>;
}
