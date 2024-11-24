use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "volc",
    "Volcanic Wyrm",
    color: Color::RED,
    cost: 7,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 500,
    shards: 6,
);

impl Effect for CardDef {
    fn event_filter(&self) -> EventFilter {
        EventFilter::ATTACKING | EventFilter::BLOCKING
    }

    fn trigger(&mut self, id: EffectId, ctx: &mut EffectTriggerContext) -> Result<()> {
        if id == "main" {
            ctx.push_stack("main", |ctx, _| {
                let target = ctx.state().players.next_id(ctx.source().controller())?;
                let commands = vec![ActionCommand::InflictDamage {
                    target,
                    amount: 200,
                }];
                Ok(EffectReport::default().with_commands(commands))
            });
        }
        Ok(())
    }

    fn activate(&mut self, _event: CardEvent, ctx: &mut EffectActivateContext) -> Result<()> {
        ctx.trigger_stack("main");
        Ok(())
    }
}
