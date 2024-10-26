use crate::assets::AssetServerExt;

use super::GlobalState;
use bevy::{asset::LoadState, prelude::*};
use kodecks_catalog::CATALOG;

pub struct PreloaderPlugin;

impl Plugin for PreloaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PreloaderState>().add_systems(
            Update,
            update
                .run_if(not(in_state(GlobalState::AppInit)).and_then(in_state(PreloaderState::On))),
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum PreloaderState {
    #[default]
    On,
    Off,
}

fn update(
    asset_server: Res<AssetServer>,
    mut loading: Local<Handle<Image>>,
    mut index: Local<usize>,
    mut next_state: ResMut<NextState<PreloaderState>>,
    mut timer: Local<Option<Timer>>,
    time: Res<Time<Real>>,
) {
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(0.1, TimerMode::Repeating));
    if !timer.tick(time.delta()).just_finished()
        || asset_server.load_state(loading.id()) == LoadState::Loading
    {
        return;
    }
    if let Some(archetype) = CATALOG.iter().nth(*index) {
        debug!("Preloading: {}", archetype.name);
        *loading =
            asset_server.load_with_cache(format!("cards/{}/image.main.png", archetype.safe_name));
        *index += 1;
    } else {
        info!("Preloaded {} assets", *index);
        next_state.set(PreloaderState::Off);
    }
}
