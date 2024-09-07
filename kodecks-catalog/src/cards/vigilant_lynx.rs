use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "vigi",
    "Vigilant Lynx",
    color: Color::GREEN,
    cost: 2,
    card_type: CardType::Creature,
    power: 100,
);

impl Effect for CardDef {
    fn event_filter(&self) -> EventFilter {
        EventFilter::ANY_CASTED
    }

    fn trigger(&mut self, id: EffectId, ctx: &mut EffectTriggerContext) -> Result<()> {
        if id == "main" {
            ctx.push_stack("main", |ctx, _| {
                ctx.push_continuous(CardDef, condition::InTurn(ctx.state().turn));
                Ok(EffectReport::default())
            });
        }
        Ok(())
    }

    fn activate(&mut self, _event: CardEvent, ctx: &mut EffectActivateContext) -> Result<()> {
        if ctx.source().id() != ctx.target().id()
            && ctx.source().controller() != ctx.target().controller()
        {
            ctx.trigger_stack("main");
        }
        Ok(())
    }
}

impl ContinuousEffect for CardDef {
    fn apply_card(
        &mut self,
        _state: &GameState,
        source: &Card,
        target: &Card,
        computed: &mut ComputedAttribute,
    ) -> Result<()> {
        if target.id() == source.id() {
            if let Some(power) = &mut computed.power {
                power.add(100);
            }
        }
        Ok(())
    }
}
