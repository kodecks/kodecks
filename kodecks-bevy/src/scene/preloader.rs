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
) {
    if asset_server.load_state(loading.id()) == LoadState::Loading {
        return;
    }
    if let Some(archetype) = CATALOG.iter().nth(*index) {
        debug!("Preloading: {}", archetype.name);
        *loading = asset_server.load(format!("cards/{}/image.main.png", archetype.safe_name));
        *index += 1;
    } else {
        info!("Preloaded {} assets", *index);
        next_state.set(PreloaderState::Off);
    }
}
