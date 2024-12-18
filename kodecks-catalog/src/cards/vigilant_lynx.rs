use crate::card_def;
use kodecks::{prelude::*, target::Target};

card_def!(
    CardDef,
    "vigi",
    "Vigilant Lynx",
    color: Color::GREEN,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 1,
    shards: 1,
);

impl Effect for CardDef {
    fn event_filter(&self) -> EventFilter {
        EventFilter::ANY_CASTED
    }

    fn trigger(&mut self, id: EffectId, ctx: &mut EffectTriggerContext) -> Result<()> {
        if id == "main" {
            ctx.push_stack("main", |ctx, _| {
                ctx.push_continuous(
                    Powerup {
                        turn: ctx.state().turn,
                    },
                    Target::Card(ctx.source().id()),
                );
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

#[derive(Clone)]
struct Powerup {
    turn: u16,
}

impl ContinuousEffect for Powerup {
    fn apply_card(&mut self, ctx: &mut ContinuousCardEffectContext) -> Result<bool> {
        if ctx.state.turn != self.turn {
            return Ok(false);
        }
        if ctx.target.id() == ctx.source.id() {
            if let Some(power) = &mut ctx.computed.power {
                power.add(100);
            }
        }
        Ok(true)
    }
}
