use super::{spinner::SpinnerState, translator::Translator, GlobalState};
use crate::{
    assets::{
        card::RenderedCardPlugin,
        fluent::{FluentAsset, FluentPlugin, DEFAULT_LANG},
    },
    config::{ConfigPlugin, GlobalConfig},
    scene::lang::find_language,
};
use bevy::{asset::LoadState, prelude::*};
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
            .add_systems(Update, update.run_if(in_state(GlobalState::AppInit)))
            .add_systems(OnExit(GlobalState::AppInit), cleanup);
    }
}

fn init(mut next_spinner_state: ResMut<NextState<SpinnerState>>) {
    info!("{}", build_info::format!("{}", $));
    next_spinner_state.set(SpinnerState::On);
}

fn cleanup(mut next_spinner_state: ResMut<NextState<SpinnerState>>) {
    next_spinner_state.set(SpinnerState::Off);
}

fn update(
    mut commands: Commands,
    config: Res<GlobalConfig>,
    mut preloading_assets: Local<Option<Vec<UntypedHandle>>>,
    asset_server: Res<AssetServer>,
    mut fluent: Local<Option<Handle<FluentAsset>>>,
    fluent_assets: Res<Assets<FluentAsset>>,
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

    let preloading_assets = preloading_assets.get_or_insert_with(|| {
        vec![
            asset_server.load::<Image>("ui/button.png").untyped(),
            asset_server.load::<Image>("ui/button-red.png").untyped(),
        ]
    });

    preloading_assets
        .retain(|handle| asset_server.get_load_state(handle) == Some(LoadState::Loading));

    if preloading_assets.is_empty() {
        if let Some(res) = fluent_assets.get(fluent) {
            commands.insert_resource(Translator::new(res));
            next_state.set(GlobalState::MenuMain);
        }
    }
}
