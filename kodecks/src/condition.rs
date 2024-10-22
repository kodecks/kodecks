use crate::{env::GameState, id::ObjectId, zone::Zone};
use dyn_clone::DynClone;

pub trait Condition: DynClone + Send + Sync {
    fn is_met(&self, state: &GameState) -> bool;
    fn or<T: Condition>(self, other: T) -> Or<Self, T>
    where
        Self: Sized,
    {
        Or(self, other)
    }
    fn and<T: Condition>(self, other: T) -> And<Self, T>
    where
        Self: Sized,
    {
        And(self, other)
    }
}

dyn_clone::clone_trait_object!(Condition);

#[derive(Debug, Clone, Copy)]
pub struct OnField(pub ObjectId);

impl Condition for OnField {
    fn is_met(&self, state: &GameState) -> bool {
        state
            .find_zone(self.0)
            .map_or(false, |zone| zone.zone == Zone::Field)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InTurn(pub u16);

impl Condition for InTurn {
    fn is_met(&self, state: &GameState) -> bool {
        state.turn == self.0
    }
}

#[derive(Debug)]
pub struct Or<A: Condition, B: Condition>(pub A, pub B);

impl<A: Condition, B: Condition> Clone for Or<A, B> {
    fn clone(&self) -> Self {
        Self(dyn_clone::clone(&self.0), dyn_clone::clone(&self.1))
    }
}

impl<A: Condition, B: Condition> Condition for Or<A, B> {
    fn is_met(&self, state: &GameState) -> bool {
        self.0.is_met(state) || self.1.is_met(state)
    }
}

#[derive(Debug)]
pub struct And<A: Condition, B: Condition>(pub A, pub B);

impl<A: Condition, B: Condition> Clone for And<A, B> {
    fn clone(&self) -> Self {
        Self(dyn_clone::clone(&self.0), dyn_clone::clone(&self.1))
    }
}

impl<A: Condition, B: Condition> Condition for And<A, B> {
    fn is_met(&self, state: &GameState) -> bool {
        self.0.is_met(state) && self.1.is_met(state)
    }
}
