use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use card::Catalog;
use kodecks_catalog::CATALOG;

pub mod app;
pub mod card;
pub mod deck;
pub mod game;
pub mod lang;
pub mod menu;
pub mod preloader;
pub mod spinner;
pub mod translator;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum GlobalState {
    #[default]
    AppInit,
    MenuMain,
    GameInit,
    GameLoading,
    GameMain,
    GameResult,
    GameCleanup,
    DeckMain,
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Catalog::new(CATALOG.clone()))
            .add_plugins(DefaultPickingPlugins)
            .add_plugins(app::AppLoadingPlugin)
            .add_plugins(menu::MenuPlugin)
            .add_plugins(deck::DeckPlugin)
            .add_plugins(spinner::SpinnerPlugin)
            .add_plugins(preloader::PreloaderPlugin)
            .add_plugins(game::loading::GameLoadingPlugin)
            .add_plugins(game::main::GameMainPlugin)
            .add_plugins(game::result::GameResultPlugin)
            .add_plugins(game::cleanup::GameCleanupPlugin)
            .add_plugins(game::camera::CameraPlugin);
    }
}
