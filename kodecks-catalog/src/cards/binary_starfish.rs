use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "bina",
    "Binary Starfish",
    color: Color::BLUE,
    cost: 3,
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
                let token = ctx.new_id();
                let commands = vec![ActionCommand::GenerateCardToken {
                    token,
                    archetype: ctx.source().archetype().id,
                    player: ctx.source().controller(),
                }];
                Ok(EffectReport::default().with_commands(commands))
            });
        }
        Ok(())
    }

    fn activate(&mut self, event: CardEvent, ctx: &mut EffectActivateContext) -> Result<()> {
        if let CardEvent::Casted { from } = event {
            if from.zone == Zone::Hand {
                ctx.trigger_stack("main");
            }
        }
        Ok(())
    }
}
