use crate::scene::GlobalState;
use bevy::prelude::*;

pub struct GameCleanupPlugin;

impl Plugin for GameCleanupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalState::GameCleanup), init);
    }
}

fn init(mut next_state: ResMut<NextState<GlobalState>>) {
    next_state.set(GlobalState::MenuMain);
}
