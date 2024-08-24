use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "bamb",
    "Bambooster",
    color: Color::RUBY,
    cost: 1,
    card_type: CardType::Creature,
    power: 300,
);

impl Effect for CardDef {
    fn event_filter(&self) -> EventFilter {
        EventFilter::ATTACKING
    }

    fn trigger(&mut self, id: EffectId, ctx: &mut EffectTriggerContext) -> Result<()> {
        if id == "main" {
            ctx.push_stack("main", |ctx, _| {
                let target = ctx.source().controller();
                let power = ctx
                    .source()
                    .computed()
                    .power
                    .map_or(0, |power| power.value());
                let commands = if power > 0 {
                    vec![ActionCommand::InflictDamage {
                        target,
                        damage: power,
                    }]
                } else {
                    vec![]
                };
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
