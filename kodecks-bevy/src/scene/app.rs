use super::{
    config::{ConfigPlugin, GlobalConfig},
    translator::Translator,
    GlobalState,
};
use crate::assets::{
    card::RenderedCardPlugin,
    fluent::{FluentAsset, FluentPlugin, DEFAULT_LANG},
};
use bevy::prelude::*;

pub struct AppLoadingPlugin;

impl Plugin for AppLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GlobalState>()
            .add_plugins(ConfigPlugin)
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
    let lang = config.lang.clone().unwrap_or(DEFAULT_LANG);
    let path = format!("locales/{lang}/lang.json");
    let fluent = fluent.get_or_insert_with(|| asset_server.load::<FluentAsset>(path));
    if let Some(res) = assets.get(fluent) {
        commands.insert_resource(Translator::new(res));
        next_state.set(GlobalState::MenuMain);
    }
}
