use crate::input::UserAction;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use leafwing_input_manager::prelude::*;

pub struct DebuggerPlugin;

impl Plugin for DebuggerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<DebuggerState>()
            .add_plugins(WorldInspectorPlugin::new().run_if(in_state(DebuggerState::Active)))
            .add_systems(Update, toggle_inspector);
    }
}

#[derive(Debug, Copy, Clone, Default, States, Eq, PartialEq, Hash)]
enum DebuggerState {
    #[default]
    Inactive,
    Active,
}

fn toggle_inspector(
    action_query: Query<&ActionState<UserAction>>,
    state: Res<State<DebuggerState>>,
    mut next_state: ResMut<NextState<DebuggerState>>,
) {
    let action_state = action_query.single();
    if action_state.just_pressed(&UserAction::ToggleDebugger) {
        next_state.set(if *state == DebuggerState::Active {
            DebuggerState::Inactive
        } else {
            DebuggerState::Active
        });
    }
}
