use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<UserAction>::default())
            .add_systems(Startup, init);
    }
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum UserAction {
    Attack,
    AllAttack,
    Block,
    Continue,
    Concede,
}

fn init(mut commands: Commands) {
    let mut input_map = InputMap::new([
        (UserAction::Attack, KeyCode::KeyA),
        (UserAction::Block, KeyCode::KeyA),
        (UserAction::Continue, KeyCode::Space),
    ]);
    input_map.insert(
        UserAction::AllAttack,
        ButtonlikeChord::new([KeyCode::ShiftLeft, KeyCode::KeyA]),
    );
    input_map.insert(
        UserAction::Concede,
        ButtonlikeChord::new([KeyCode::ShiftLeft, KeyCode::ControlLeft, KeyCode::KeyO]),
    );
    commands.spawn(InputManagerBundle::with_map(input_map));
}
