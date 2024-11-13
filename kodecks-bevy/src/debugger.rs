use crate::input::UserAction;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiPlugin},
    bevy_inspector, egui,
};
use egui_tracing::EventCollector;
use leafwing_input_manager::prelude::*;

#[derive(Debug, Resource, Deref)]
pub struct LogCollector(EventCollector);

impl LogCollector {
    pub fn new(collector: EventCollector) -> Self {
        Self(collector)
    }
}

pub struct DebuggerPlugin;

impl Plugin for DebuggerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<DebuggerState>()
            .add_plugins(EguiPlugin)
            .add_plugins(bevy_inspector_egui::DefaultInspectorConfigPlugin) // adds default options and `InspectorEguiImpl`s
            .add_systems(
                Update,
                (
                    toggle_inspector,
                    inspector_ui.run_if(in_state(DebuggerState::Active)),
                ),
            );
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

fn inspector_ui(world: &mut World) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    let collector = world
        .get_resource::<LogCollector>()
        .map(|collector| collector.0.clone());

    egui::Window::new("Entities").show(egui_context.get_mut(), move |ui| {
        bevy_inspector::ui_for_world_entities(world, ui);
    });

    egui::Window::new("Log").show(egui_context.get_mut(), move |ui| {
        if let Some(collector) = collector {
            ui.add(egui_tracing::Logs::new(collector));
        }
    });
}
