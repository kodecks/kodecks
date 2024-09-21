use bevy::prelude::*;

mod animation;
mod battle;
mod card;
mod deck;
mod dialog;
mod event;
mod life;
mod pointer;
mod setup;
mod shard;
mod stack;
mod turn;
mod ui;

pub use animation::AnimationState;

pub struct GameMainPlugin;

impl Plugin for GameMainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(deck::DeckPlugin)
            .add_plugins(battle::BattlePlugin)
            .add_plugins(life::LifePlugin)
            .add_plugins(shard::ShardPlugin)
            .add_plugins(animation::AnimationPlugin)
            .add_plugins(event::EventPlugin)
            .add_plugins(card::CardPlugin)
            .add_plugins(ui::UiPlugin)
            .add_plugins(stack::StackPlugin)
            .add_plugins(pointer::PointerPlugin)
            .add_plugins(dialog::DialogPlugin)
            .add_plugins(turn::TurnPlugin)
            .add_systems(Startup, setup::setup);
    }
}
