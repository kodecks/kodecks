use crate::{
    action::{Action, PlayerAvailableActions},
    card::Card,
    command::ActionCommand,
    continuous::{ContinuousEffect, ContinuousItem},
    dsl::script::{
        error::Error,
        exp::{ExpEnv, ExpParams, Module},
        value::{CustomType, Value},
    },
    env::GameState,
    event::{CardEvent, EventFilter},
    id::{CardId, ObjectId, ObjectIdCounter, TimedCardId},
    player::{Player, PlayerList},
    prelude::{ComputedAttribute, ComputedAttributeModifier},
    stack::StackItem,
    target::Target,
};
use bincode::{
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{DecodeError, EncodeError},
    BorrowDecode, Decode, Encode,
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

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct EffectReport {
    #[serde(default)]
    pub available_actions: Option<PlayerAvailableActions>,
    #[serde(default)]
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

impl Encode for EffectId {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(self.0.as_str(), encoder)?;
        Ok(())
    }
}

impl Decode for EffectId {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self(TinyAsciiStr::from_bytes_lossy(
            <String as Decode>::decode(decoder)?.as_bytes(),
        )))
    }
}

impl<'de> BorrowDecode<'de> for EffectId {
    fn borrow_decode<D: BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self(TinyAsciiStr::from_bytes_lossy(
            <String as Decode>::decode(decoder)?.as_bytes(),
        )))
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
    obj_counter: &'a mut ObjectIdCounter,
}

impl<'a> EffectTriggerContext<'a> {
    pub fn new(
        state: &'a GameState,
        obj_counter: &'a mut ObjectIdCounter,
        source: &'a Card,
    ) -> Self {
        Self {
            state,
            source,
            continuous: Vec::new(),
            stack: Vec::new(),
            obj_counter,
        }
    }

    pub fn state(&self) -> &GameState {
        self.state
    }

    pub fn source(&self) -> &Card {
        self.source
    }

    pub fn push_continuous<F>(&mut self, effect: F, target: Target)
    where
        F: ContinuousEffect + 'static,
    {
        self.continuous
            .push(ContinuousItem::new(self.source, effect, target));
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

    pub fn new_id(&mut self) -> ObjectId {
        self.obj_counter.allocate(None)
    }

    pub fn into_inner(self) -> (Vec<ContinuousItem>, Vec<StackItem>) {
        (self.continuous, self.stack)
    }
}

impl ExpEnv for EffectTriggerContext<'_> {
    fn get_var(&self, name: &str) -> Option<Value> {
        match name {
            "$source" => Some(self.source.timed_id().into()),
            _ => self.state.get_var(name),
        }
    }

    fn get_card<T>(&self, id: T) -> Option<&Card>
    where
        T: CardId + Copy,
    {
        self.state.find_card(id).ok()
    }

    fn get_players(&self) -> Option<&PlayerList<Player>> {
        Some(self.state.players())
    }

    fn invoke(
        &mut self,
        name: &str,
        args: Vec<Value>,
        params: &ExpParams,
        input: &Value,
    ) -> Result<Vec<Value>, Error> {
        match name {
            "push_stack" => {
                if args.is_empty() {
                    return Err(Error::InvalidArgumentCount);
                }
                let id = args[0].to_string();
                let params = Arc::new(params.clone());
                let args = args.into_iter().skip(1).collect::<Vec<_>>();
                self.push_stack(&id.clone(), move |ctx, action| {
                    let event: Value = if let Some(action) = action {
                        action.into()
                    } else {
                        Default::default()
                    };
                    let mut new_args: Vec<Value> = vec![id.clone().into(), event];
                    new_args.extend(args.clone());
                    let module = Module::new(params.clone());
                    let report: Option<EffectReport> = module.call(ctx, "stack", new_args)?;
                    Ok(report.unwrap_or_default())
                });
                Ok(vec![input.clone()])
            }
            "push_continuous" => {
                if args.is_empty() {
                    return Err(Error::InvalidArgumentCount);
                }
                let target = match args[1] {
                    Value::Custom(CustomType::Card(card)) => Target::Card(card.id),
                    Value::Custom(CustomType::Player(player)) => Target::Player(player),
                    _ => return Err(Error::InvalidConversion),
                };
                let params = Arc::new(params.clone());
                let module = Module::new(params.clone());
                let args = args
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, v)| if i == 1 { None } else { Some(v) })
                    .collect::<Vec<_>>();
                let effect = ModuleContinuousEffect { module, args };
                self.push_continuous(effect, target);
                Ok(vec![input.clone()])
            }
            _ => self.state.invoke(name, args, params, input),
        }
    }
}

pub struct ContinuousCardEffectContext<'a> {
    pub state: &'a GameState,
    pub source: &'a Card,
    pub target: &'a Card,
    pub computed: &'a mut ComputedAttribute,
}

impl ExpEnv for ContinuousCardEffectContext<'_> {
    fn get_var(&self, name: &str) -> Option<Value> {
        match name {
            "$source" => Some(self.source.timed_id().into()),
            "$target" => Some(self.target.timed_id().into()),
            _ => self.state.get_var(name),
        }
    }

    fn get_card<T>(&self, id: T) -> Option<&Card>
    where
        T: CardId + Copy,
    {
        self.state.find_card(id).ok()
    }

    fn get_players(&self) -> Option<&PlayerList<Player>> {
        Some(&self.state.players)
    }

    fn invoke(
        &mut self,
        name: &str,
        args: Vec<Value>,
        params: &ExpParams,
        input: &Value,
    ) -> Result<Vec<Value>, Error> {
        self.state.invoke(name, args, params, input)
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

impl ExpEnv for EffectActivateContext<'_> {
    fn get_var(&self, name: &str) -> Option<Value> {
        match name {
            "$source" => Some(self.source.timed_id().into()),
            "$target" => Some(self.target.timed_id().into()),
            _ => self.state.get_var(name),
        }
    }

    fn get_card<T>(&self, id: T) -> Option<&Card>
    where
        T: CardId + Copy,
    {
        self.state.find_card(id).ok()
    }

    fn get_players(&self) -> Option<&PlayerList<Player>> {
        Some(&self.state.players)
    }

    fn invoke(
        &mut self,
        name: &str,
        args: Vec<Value>,
        params: &ExpParams,
        input: &Value,
    ) -> Result<Vec<Value>, Error> {
        match name {
            "trigger_stack" => {
                if args.len() != 1 {
                    return Err(Error::InvalidArgumentCount);
                }
                let id = args[0].to_string();
                self.trigger_stack(id);
                Ok(vec![input.clone()])
            }
            _ => self.state.invoke(name, args, params, input),
        }
    }
}

pub trait Effect: Send + Sync + DynClone {
    fn event_filter(&self) -> EventFilter {
        EventFilter::empty()
    }

    fn is_castable(&self, _state: &GameState, _target: &Card, castable: bool) -> bool {
        castable
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
#[derive(Debug, Clone)]
struct ModuleContinuousEffect {
    module: Module,
    args: Vec<Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ModifierOrBool {
    Bool(bool),
    Modifier(ComputedAttributeModifier),
}

impl ContinuousEffect for ModuleContinuousEffect {
    fn apply_card(&mut self, ctx: &mut ContinuousCardEffectContext) -> anyhow::Result<bool> {
        let modifier: Option<ModifierOrBool> =
            self.module.call(ctx, "continuous", self.args.clone())?;
        if let Some(modifier) = modifier {
            match modifier {
                ModifierOrBool::Modifier(modifier) => {
                    ctx.computed.apply_modifier(modifier);
                }
                ModifierOrBool::Bool(value) => {
                    return Ok(value);
                }
            }
        }
        Ok(true)
    }
}
