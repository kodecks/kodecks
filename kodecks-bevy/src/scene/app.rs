use super::{
    config::{ConfigPlugin, GlobalConfig},
    translator::Translator,
    GlobalState,
};
use crate::{
    assets::{
        card::RenderedCardPlugin,
        fluent::{FluentAsset, FluentPlugin, DEFAULT_LANG},
    },
    scene::lang::find_language,
};
use bevy::prelude::*;
use std::str::FromStr;
use unic_langid::LanguageIdentifier;

build_info::build_info!(fn build_info);

pub struct AppLoadingPlugin;

impl Plugin for AppLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GlobalState>()
            .add_plugins(ConfigPlugin)
            .add_plugins(RenderedCardPlugin)
            .add_plugins(FluentPlugin)
            .add_systems(Startup, init)
            .add_systems(Update, update.run_if(in_state(GlobalState::AppInit)));
    }
}

fn init() {
    info!("{}", build_info::format!("{}", $));
}

fn update(
    mut commands: Commands,
    config: Res<GlobalConfig>,
    mut fluent: Local<Option<Handle<FluentAsset>>>,
    asset_server: Res<AssetServer>,
    assets: ResMut<Assets<FluentAsset>>,
    mut next_state: ResMut<NextState<GlobalState>>,
) {
    let fluent = fluent.get_or_insert_with(|| {
        let locale =
            sys_locale::get_locale().and_then(|locale| LanguageIdentifier::from_str(&locale).ok());
        info!("Detected locale: {:?}", locale);

        let lang = config.lang.clone().or(locale).unwrap_or(DEFAULT_LANG);
        info!("Selected language: {:?}", lang);

        let lang = find_language(lang);
        info!("Found language: {} ({})", lang.name, lang.id);

        let path = format!("locales/{}/lang.json", lang.id);
        asset_server.load::<FluentAsset>(path)
    });

    if let Some(res) = assets.get(fluent) {
        commands.insert_resource(Translator::new(res));
        next_state.set(GlobalState::MenuMain);
    }
}
