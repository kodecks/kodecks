use bevy::prelude::*;
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
