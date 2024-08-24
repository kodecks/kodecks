use super::{config::GlobalConfig, translator::Translator, GlobalState};
use crate::assets::{
    card::RenderedCardPlugin,
    fluent::{FluentAsset, FluentPlugin},
};
use bevy::prelude::*;

pub struct AppLoadingPlugin;

impl Plugin for AppLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalConfig>()
            .init_state::<GlobalState>()
            .add_plugins(RenderedCardPlugin)
            .add_plugins(FluentPlugin)
            .add_systems(Update, update.run_if(in_state(GlobalState::AppInit)));
    }
}

fn update(
    mut commands: Commands,
    config: Res<GlobalConfig>,
    mut fluent: Local<Option<Handle<FluentAsset>>>,
    asset_server: Res<AssetServer>,
    assets: ResMut<Assets<FluentAsset>>,
    mut next_state: ResMut<NextState<GlobalState>>,
) {
    let path = format!("locales/{}/lang.json", config.lang);
    let fluent = fluent.get_or_insert_with(|| asset_server.load::<FluentAsset>(path));
    if let Some(res) = assets.get(fluent) {
        commands.insert_resource(Translator::new(res));
        next_state.set(GlobalState::MenuMain);
    }
}
