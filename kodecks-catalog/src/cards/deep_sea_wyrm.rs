use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "deep",
    "Deep-Sea Wyrm",
    color: Color::AZURE,
    cost: 6,
    card_type: CardType::Creature,
    power: 500,
    abilities: &[KeywordAbility::Stealth][..],
);

impl Effect for CardDef {
    fn event_filter(&self) -> EventFilter {
        EventFilter::CASTED
    }

    fn trigger(&mut self, id: EffectId, ctx: &mut EffectTriggerContext) -> Result<()> {
        if id == "main" {
            ctx.push_stack("main", |ctx, _| {
                let power = ctx
                    .source()
                    .computed()
                    .power
                    .map(|p| p.value())
                    .unwrap_or(0);
                let commands = ctx
                    .state()
                    .players()
                    .iter()
                    .flat_map(|p| p.field.iter())
                    .filter(|card| card.computed().power.map(|p| p.value()).unwrap_or(0) < power)
                    .map(|card| ActionCommand::ShuffleCardIntoDeck {
                        source: ctx.source().id(),
                        target: card.timed_id(),
                    });
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
