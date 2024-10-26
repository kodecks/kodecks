use bevy::{
    asset::{AssetPath, LoadState},
    prelude::*,
};
use dashmap::DashMap;
use std::hash::Hash;

pub mod card;
pub mod fluent;

#[derive(Resource)]
pub struct AssetHandleStore<T, A: Asset> {
    map: DashMap<T, Handle<A>>,
}

impl<T, A> Default for AssetHandleStore<T, A>
where
    T: Eq + Hash + Clone + Copy,
    A: Asset,
{
    fn default() -> Self {
        Self {
            map: DashMap::new(),
        }
    }
}

impl<T, A> AssetHandleStore<T, A>
where
    T: Eq + Hash + Clone + Copy,
    A: Asset,
{
    pub fn get(&self, key: T, assets: &Assets<A>) -> Handle<A> {
        self.map
            .entry(key)
            .or_insert_with(|| assets.reserve_handle())
            .value()
            .clone()
    }
}

pub trait AssetServerExt {
    fn load_with_cache<'a, A>(&self, path: impl Into<AssetPath<'a>>) -> Handle<A>
    where
        A: Asset;
}

impl AssetServerExt for AssetServer {
    fn load_with_cache<'a, A>(&self, path: impl Into<AssetPath<'a>>) -> Handle<A>
    where
        A: Asset,
    {
        let path: AssetPath<'a> = path.into();
        if let Some(handle) = self.get_handle(path.clone()) {
            if self.get_load_state(&handle.clone().untyped()) == Some(LoadState::Loaded) {
                return handle;
            }
        }
        
        self.load(path.clone())
    }
}
