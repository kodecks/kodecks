use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "mire",
    "Mire Alligator",
    color: Color::JADE,
    cost: 3,
    card_type: CardType::Creature,
    power: 400,
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
                    .map(|card| card.id())
                    .collect::<Vec<_>>();
                if candidates.is_empty() {
                    return Ok(EffectReport::default());
                }
                if let Some(Action::SelectCard { card }) = action {
                    let commands = vec![ActionCommand::DestroyCard {
                        source: card,
                        target: card,
                        reason: EventReason::Effect,
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
                        message_dialog: Some(MessageDialog {
                            messages: vec![MessageBox {
                                message: Message {
                                    id: "mire_alligator".to_string(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
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
