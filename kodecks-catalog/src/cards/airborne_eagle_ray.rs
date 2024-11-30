use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "airb",
    "Airborne Eagle Ray",
    color: Color::BLUE,
    cost: 3,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 300,
);

impl Effect for CardDef {
    fn event_filter(&self) -> EventFilter {
        EventFilter::CASTED
    }

    fn trigger(&mut self, id: EffectId, ctx: &mut EffectTriggerContext) -> Result<()> {
        if id == "main" {
            ctx.push_stack("main", |ctx, action| {
                let controller = ctx.source().controller();
                let candidates = ctx
                    .state()
                    .players()
                    .iter()
                    .flat_map(|player| player.field.iter())
                    .filter(|card| card.flags().is_targetable())
                    .map(|card| card.timed_id())
                    .collect::<Vec<_>>();
                if candidates.is_empty() {
                    return Ok(EffectReport::default());
                }
                if let Some(Action::SelectCard { card }) = action {
                    let target = ctx.state().find_card(card)?;
                    let commands = vec![ActionCommand::ReturnCardToHand {
                        source: ctx.source().id(),
                        target: target.timed_id(),
                        reason: EventReason::Effect,
                    }];
                    return Ok(EffectReport::default().with_commands(commands));
                }
                Ok(
                    EffectReport::default().with_available_actions(PlayerAvailableActions {
                        player: controller,
                        actions: vec![AvailableAction::SelectCard { cards: candidates }]
                            .into_iter()
                            .collect(),
                        instructions: Some(Message {
                            id: "card-airborne-eagle-ray.message".to_string(),
                            ..Default::default()
                        }),
                        message_dialog: None,
                    }),
                )
            });
        }
        Ok(())
    }

    fn activate(&mut self, _event: CardEvent, ctx: &mut EffectActivateContext) -> Result<()> {
        ctx.trigger_stack("main");
        Ok(())
    }
}
