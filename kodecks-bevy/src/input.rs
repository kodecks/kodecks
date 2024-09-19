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
}

fn init(mut commands: Commands) {
    let mut input_map = InputMap::new([
        (UserAction::Attack, KeyCode::KeyA),
        (UserAction::Block, KeyCode::KeyA),
        (UserAction::Continue, KeyCode::Space),
    ]);
    input_map.insert_chord(UserAction::AllAttack, [KeyCode::ShiftLeft, KeyCode::KeyA]);
    commands.spawn(InputManagerBundle::with_map(input_map));
}
