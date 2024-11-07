use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "pyro",
    "Pyrosnail",
    color: Color::RED,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 100,
    abilities: &[KeywordAbility::Volatile][..],
);

impl Effect for CardDef {
    fn event_filter(&self) -> EventFilter {
        EventFilter::DESTROYED
    }

    fn trigger(&mut self, id: EffectId, ctx: &mut EffectTriggerContext) -> Result<()> {
        if id == "main" {
            ctx.push_stack("main", |ctx, _| {
                let target = ctx.state().players.next_id(ctx.source().controller())?;
                let commands = vec![ActionCommand::InflictDamage {
                    target,
                    amount: 100,
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
