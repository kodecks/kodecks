use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "flas",
    "Flash-Bang Jellyfish",
    color: Color::AZURE,
    cost: 3,
    card_type: CardType::Creature,
    power: 300,
);

impl Effect for CardDef {
    fn event_filter(&self) -> EventFilter {
        EventFilter::CASTED
    }

    fn trigger(&mut self, id: EffectId, ctx: &mut EffectTriggerContext) -> Result<()> {
        if id == "main" {
            ctx.push_stack("main", |ctx, _| {
                let commands = ctx
                    .state()
                    .players()
                    .iter()
                    .flat_map(|p| p.field.iter())
                    .map(|card| ActionCommand::SetFieldState {
                        source: ctx.source().id(),
                        target: card.id(),
                        state: kodecks::field::FieldState::Exhausted,
                        reason: EventReason::Effect,
                    });
                Ok(EffectReport::default().with_commands(commands))
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
