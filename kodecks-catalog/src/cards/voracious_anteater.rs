use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "vora",
    "Voracious Anteater",
    color: Color::GREEN,
    cost: 3,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 400,
    abilities: &[KeywordAbility::Devour][..],
    shards: 1,
);

impl Effect for CardDef {
    fn event_filter(&self) -> EventFilter {
        EventFilter::DESTROYED
    }

    fn trigger(&mut self, id: EffectId, ctx: &mut EffectTriggerContext) -> Result<()> {
        if id == "main" {
            ctx.push_stack("main", |ctx, _| {
                let token = ctx.new_id();
                let commands = vec![ActionCommand::GenerateCardToken {
                    token,
                    archetype: ArchetypeId::new("ant"),
                    player: ctx.source().controller(),
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
