use super::GlobalState;
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GlobalState::MenuMain), init);
    }
}

fn init(mut next_state: ResMut<NextState<GlobalState>>) {
    next_state.set(GlobalState::GameInit);
}
