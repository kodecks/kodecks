use crate::{
    action::{Action, PlayerAvailableActions},
    card::Card,
    command::ActionCommand,
    condition::Condition,
    continuous::{ContinuousEffect, ContinuousItem},
    env::GameState,
    event::{CardEvent, EventFilter},
    stack::StackItem,
};
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug},
    sync::Arc,
};
use tinystr::TinyAsciiStr;

pub type StackEffectHandler = dyn Fn(&mut EffectTriggerContext, Option<Action>) -> anyhow::Result<EffectReport>
    + Send
    + Sync
    + 'static;

#[derive(Debug, Default)]
pub struct EffectReport {
    pub available_actions: Option<PlayerAvailableActions>,
    pub commands: Vec<ActionCommand>,
}

impl EffectReport {
    pub fn with_commands<I>(mut self, commands: I) -> Self
    where
        I: IntoIterator<Item = ActionCommand>,
    {
        self.commands.extend(commands);
        self
    }

    pub fn with_available_actions(mut self, available_actions: PlayerAvailableActions) -> Self {
        self.available_actions = Some(available_actions);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NoEffect;

impl Effect for NoEffect {}

impl NoEffect {
    pub const NEW: fn() -> Box<dyn Effect> = || Box::new(NoEffect);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EffectId(TinyAsciiStr<16>);

impl fmt::Display for EffectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl EffectId {
    pub fn new(id: &str) -> Self {
        Self(TinyAsciiStr::from_bytes_lossy(id.as_bytes()))
    }
}

impl<T> From<T> for EffectId
where
    T: AsRef<str>,
{
    fn from(id: T) -> Self {
        Self::new(id.as_ref())
    }
}

impl<T> PartialEq<T> for EffectId
where
    T: AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

pub struct EffectTriggerContext<'a> {
    state: &'a GameState,
    source: &'a Card,
    continuous: Vec<ContinuousItem>,
    stack: Vec<StackItem>,
}

impl<'a> EffectTriggerContext<'a> {
    pub fn new(state: &'a GameState, source: &'a Card) -> Self {
        Self {
            state,
            source,
            continuous: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn state(&self) -> &GameState {
        self.state
    }

    pub fn source(&self) -> &Card {
        self.source
    }

    pub fn push_continuous<F, C>(&mut self, effect: F, condition: C)
    where
        F: ContinuousEffect + 'static,
        C: Condition + 'static,
    {
        self.continuous
            .push(ContinuousItem::new(self.source, effect, condition));
    }

    pub fn push_stack<F>(&mut self, id: &str, handler: F)
    where
        F: Fn(&mut EffectTriggerContext, Option<Action>) -> anyhow::Result<EffectReport>
            + Send
            + Sync
            + 'static,
    {
        self.stack.push(StackItem {
            source: self.source.id(),
            id: id.to_string(),
            handler: Arc::new(Box::new(handler)),
        });
    }

    pub fn into_inner(self) -> (Vec<ContinuousItem>, Vec<StackItem>) {
        (self.continuous, self.stack)
    }
}

pub struct EffectActivateContext<'a> {
    state: &'a GameState,
    source: &'a Card,
    target: &'a Card,
    continuous: Vec<EffectId>,
    stack: Vec<EffectId>,
}

impl<'a> EffectActivateContext<'a> {
    pub fn new(state: &'a GameState, source: &'a Card, target: &'a Card) -> Self {
        Self {
            state,
            source,
            target,
            continuous: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn state(&self) -> &GameState {
        self.state
    }

    pub fn source(&self) -> &Card {
        self.source
    }

    pub fn target(&self) -> &Card {
        self.target
    }

    pub fn trigger_continuous<T>(&mut self, id: T)
    where
        T: Into<EffectId>,
    {
        self.continuous.push(id.into());
    }

    pub fn trigger_stack<T>(&mut self, id: T)
    where
        T: Into<EffectId>,
    {
        self.stack.push(id.into());
    }

    pub fn into_inner(self) -> (Vec<EffectId>, Vec<EffectId>) {
        (self.continuous, self.stack)
    }
}

pub trait Effect: Send + Sync + DynClone {
    fn event_filter(&self) -> EventFilter {
        EventFilter::empty()
    }

    fn is_castable(&self, _state: &GameState, _target: &Card) -> bool {
        true
    }

    fn trigger(&mut self, _id: EffectId, _ctx: &mut EffectTriggerContext) -> anyhow::Result<()> {
        Ok(())
    }

    fn activate(
        &mut self,
        _event: CardEvent,
        _ctx: &mut EffectActivateContext,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
