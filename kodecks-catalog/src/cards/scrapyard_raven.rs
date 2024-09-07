use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "scra",
    "Scrapyard Raven",
    color: Color::GREEN,
    cost: 2,
    card_type: CardType::Creature,
    power: 200,
);

impl Effect for CardDef {
    fn event_filter(&self) -> EventFilter {
        EventFilter::CASTED
    }

    fn trigger(&mut self, id: EffectId, ctx: &mut EffectTriggerContext) -> Result<()> {
        if id == "main" {
            ctx.push_stack("main", |ctx, _| {
                Ok(
                    EffectReport::default().with_commands(vec![ActionCommand::GenerateShards {
                        player: ctx.source().zone().player,
                        source: ctx.source().id(),
                        color: ctx.source().computed().color,
                        amount: 1,
                    }]),
                )
            });
        }
        Ok(())
    }

    fn activate(&mut self, _event: CardEvent, ctx: &mut EffectActivateContext) -> Result<()> {
        let target = ctx.source().zone().player;
        if ctx.state().players().get(target).shards.is_empty() {
            ctx.trigger_stack("main");
        }
        Ok(())
    }
}
