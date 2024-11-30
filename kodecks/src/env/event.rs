use super::Environment;
use crate::{
    ability::KeywordAbility,
    card::Card,
    color::Color,
    error::ActionError,
    event::CardEvent,
    filter_vec,
    opcode::{Opcode, OpcodeList},
    player::Zone,
    zone::{CardZone, MoveReason, ZoneKind},
};

impl Environment {
    pub fn apply_event(
        &self,
        event: CardEvent,
        source: &Card,
        target: &Card,
    ) -> Result<Vec<OpcodeList>, ActionError> {
        let trigger = if target.event_filter().contains(event.filter()) {
            Some(Opcode::TriggerEvent {
                source: source.id(),
                target: target.id(),
                event,
            })
        } else {
            None
        };

        match event {
            CardEvent::Destroyed { .. } => {
                let from = *target.zone();
                let to = Zone::new(target.owner(), ZoneKind::Graveyard);
                let volatile = target
                    .computed()
                    .abilities
                    .contains(&KeywordAbility::Volatile);
                let devour = source
                    .computed()
                    .abilities
                    .contains(&KeywordAbility::Devour);
                let token = target.is_token();
                Ok(filter_vec![
                    if volatile || devour || token {
                        None
                    } else {
                        Some(OpcodeList::new(vec![Opcode::GenerateShards {
                            player: to.player,
                            color: Color::COLORLESS,
                            amount: 1,
                        }]))
                    },
                    Some(OpcodeList::new(filter_vec![
                        Some(Opcode::MoveCard {
                            card: target.id(),
                            from,
                            to,
                            reason: MoveReason::Destroyed,
                        }),
                        trigger,
                    ],)),
                ])
            }
            CardEvent::ReturnedToHand { .. } => {
                let from = *target.zone();
                let to = Zone::new(target.owner(), ZoneKind::Hand);
                Ok(filter_vec![Some(OpcodeList::new(filter_vec![
                    Some(Opcode::MoveCard {
                        card: target.id(),
                        from,
                        to,
                        reason: MoveReason::Move,
                    }),
                    trigger,
                ],)),])
            }
            CardEvent::ReturnedToDeck => {
                let from = *target.zone();
                let to = Zone::new(target.owner(), ZoneKind::Deck);
                Ok(filter_vec![Some(OpcodeList::new(filter_vec![
                    Some(Opcode::MoveCard {
                        card: target.id(),
                        from,
                        to,
                        reason: MoveReason::Move,
                    }),
                    trigger,
                ],)),])
            }
            _ => Ok(vec![OpcodeList::new(filter_vec![trigger,])]),
        }
    }

    pub fn apply_event_any(
        &self,
        event: CardEvent,
        source: &Card,
    ) -> Result<Vec<OpcodeList>, ActionError> {
        let mut opcodes = vec![];
        for player in self.state.players.iter() {
            for card in player.field.iter() {
                opcodes.extend(self.apply_event(event, source, card)?);
            }
        }
        Ok(opcodes)
    }
}
