use crate::dsl::script::exp::Module;
use crate::dsl::script::value::Value;
use crate::dsl::SmallStr;
use crate::effect::{Effect, EffectActivateContext};
use crate::event::{CardEvent, EventFilter};

#[derive(Debug, Clone)]
pub struct EffectDef {
    event_filter: EventFilter,
    module: Module,
}

impl EffectDef {
    pub fn new(module: Module) -> Self {
        let mut event_filter = EventFilter::empty();
        if module.has_def("on_casted", 1) {
            event_filter |= EventFilter::CASTED;
        }
        if module.has_def("on_destroyed", 1) {
            event_filter |= EventFilter::DESTROYED;
        }
        if module.has_def("on_returned_to_hand", 1) {
            event_filter |= EventFilter::RETURNED_TO_HAND;
        }
        if module.has_def("on_returned_to_deck", 1) {
            event_filter |= EventFilter::RETURNED_TO_DECK;
        }
        if module.has_def("on_dealt_damage", 1) {
            event_filter |= EventFilter::DEALT_DAMAGE;
        }
        if module.has_def("on_attacking", 1) {
            event_filter |= EventFilter::ATTACKING;
        }
        if module.has_def("on_blocking", 1) {
            event_filter |= EventFilter::BLOCKING;
        }
        if module.has_def("on_attacked", 1) {
            event_filter |= EventFilter::ATTACKED;
        }
        if module.has_def("on_any_casted", 1) {
            event_filter |= EventFilter::ANY_CASTED;
        }
        Self {
            event_filter,
            module,
        }
    }
}

impl Effect for EffectDef {
    fn event_filter(&self) -> EventFilter {
        self.event_filter
    }

    fn activate(
        &mut self,
        event: CardEvent,
        ctx: &mut EffectActivateContext,
    ) -> anyhow::Result<()> {
        let id: SmallStr = event.into();
        let name = format!("on_{}", id.as_str());
        let event: Value = event.into();
        let _: serde_json::Value = self.module.call(ctx, &name, vec![event])?;
        Ok(())
    }

    fn is_castable(
        &self,
        _state: &crate::prelude::GameState,
        _target: &crate::prelude::Card,
        castable: bool,
    ) -> bool {
        castable
    }

    fn trigger(
        &mut self,
        id: crate::prelude::EffectId,
        ctx: &mut crate::prelude::EffectTriggerContext,
    ) -> anyhow::Result<()> {
        let id: Value = id.to_string().into();
        let _: serde_json::Value = self.module.call(ctx, "trigger", vec![id])?;
        Ok(())
    }
}
