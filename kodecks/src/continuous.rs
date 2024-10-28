use crate::{
    ability::{AbilityList, PlayerAbility},
    card::Card,
    computed::ComputedAttribute,
    condition::Condition,
    env::GameState,
    id::ObjectId,
};
use core::fmt;
use dyn_clone::DynClone;
use std::sync::Arc;
use tracing::error;

#[derive(Clone)]
pub struct ContinuousItem {
    source: ObjectId,
    timestamp: u32,
    func: Arc<Box<dyn ContinuousEffect>>,
    condition: Arc<Box<dyn Condition>>,
}

impl fmt::Debug for ContinuousItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContinuousEffect")
            .field("source", &self.source)
            .field("timestamp", &self.timestamp)
            .finish()
    }
}

pub trait ContinuousEffect: Send + Sync + DynClone {
    fn apply_card(
        &mut self,
        _state: &GameState,
        _source: &Card,
        _target: &Card,
        _computed: &mut ComputedAttribute,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn apply_player(
        &mut self,
        _state: &GameState,
        _player: u8,
        _abilities: &mut AbilityList<PlayerAbility>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

dyn_clone::clone_trait_object!(ContinuousEffect);

impl ContinuousItem {
    pub fn new<F, C>(source: &Card, effect: F, condition: C) -> Self
    where
        F: ContinuousEffect + 'static,
        C: Condition + 'static,
    {
        Self {
            source: source.id(),
            timestamp: source.timestamp(),
            func: Arc::new(Box::new(effect)),
            condition: Arc::new(Box::new(condition)),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ContinuousEffectList {
    effects: Vec<ContinuousItem>,
}

impl Extend<ContinuousItem> for ContinuousEffectList {
    fn extend<T: IntoIterator<Item = ContinuousItem>>(&mut self, iter: T) {
        self.effects.extend(iter);
    }
}

impl ContinuousEffectList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, effect: ContinuousItem) {
        self.effects.push(effect);
        self.effects.sort_by_key(|effect| effect.timestamp);
    }

    pub fn apply_card(&mut self, state: &GameState, card: &Card) -> ComputedAttribute {
        let mut computed = ComputedAttribute::from(&**card.archetype());
        for effect in self.effects.iter_mut().rev() {
            if let Err(err) = state
                .find_card(effect.source)
                .map_err(|err| err.into())
                .and_then(|source| {
                    dyn_clone::arc_make_mut(&mut effect.func).apply_card(
                        state,
                        source,
                        card,
                        &mut computed,
                    )
                })
            {
                error!("Failed to apply continuous effect: {:?}", err);
            }
        }
        computed
    }

    pub fn apply_player(&mut self, state: &GameState, player: u8) -> AbilityList<PlayerAbility> {
        let mut abilities = AbilityList::new();
        for effect in self.effects.iter_mut().rev() {
            if let Err(err) = dyn_clone::arc_make_mut(&mut effect.func).apply_player(
                state,
                player,
                &mut abilities,
            ) {
                error!("Failed to apply continuous effect: {:?}", err);
            }
        }
        abilities
    }

    pub fn update(&mut self, state: &GameState) {
        self.effects.retain(|effect| effect.condition.is_met(state));
    }
}
