use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "deep",
    "Deep-Sea Wyrm",
    color: Color::BLUE,
    cost: 6,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 500,
    abilities: &[KeywordAbility::Stealth][..],
    shards: 5,
);

impl Effect for CardDef {
    fn event_filter(&self) -> EventFilter {
        EventFilter::CASTED
    }

    fn trigger(&mut self, id: EffectId, ctx: &mut EffectTriggerContext) -> Result<()> {
        if id == "main" {
            ctx.push_stack("main", |ctx, _| {
                let power = ctx.source().computed().current_power();
                let commands = ctx
                    .state()
                    .players()
                    .iter()
                    .flat_map(|p| p.field.iter())
                    .filter(|card| card.computed().current_power() < power)
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
