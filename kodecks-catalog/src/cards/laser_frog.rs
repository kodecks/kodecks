use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "lase",
    "Laser Frog",
    color: Color::RED,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 100,
);

impl Effect for CardDef {
    fn event_filter(&self) -> EventFilter {
        EventFilter::CASTED
    }

    fn trigger(&mut self, id: EffectId, ctx: &mut EffectTriggerContext) -> Result<()> {
        if id == "main" {
            ctx.push_stack("main", |ctx, action| {
                let controller = ctx.source().controller();
                let player = ctx.state().players().get(controller);
                let candidates = player
                    .field
                    .iter()
                    .filter(|card| card.flags().is_targetable())
                    .filter(|card| card.computed().current_shields() > 0)
                    .map(|card| card.id())
                    .collect::<Vec<_>>();
                if candidates.is_empty() {
                    return Ok(EffectReport::default());
                }
                if let Some(Action::SelectCard { card }) = action {
                    let target = ctx.state().find_card(card)?;
                    let commands = vec![ActionCommand::BreakShield {
                        target: target.timed_id(),
                    }];
                    return Ok(EffectReport::default().with_commands(commands));
                }
                Ok(
                    EffectReport::default().with_available_actions(PlayerAvailableActions {
                        player: controller,
                        actions: vec![AvailableAction::SelectCard {
                            cards: candidates,
                            score_factor: -1,
                        }]
                        .into_iter()
                        .collect(),
                        instructions: Some(Message {
                            id: "card-laser-frog.message".to_string(),
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
