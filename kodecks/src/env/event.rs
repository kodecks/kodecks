use super::Environment;
use crate::{
    ability::KeywordAbility,
    card::Card,
    error::Error,
    event::CardEvent,
    filter_vec,
    opcode::{Opcode, OpcodeList},
    player::PlayerZone,
    zone::{CardZone, MoveReason, Zone},
};

impl Environment {
    pub fn apply_event(
        &self,
        event: CardEvent,
        source: &Card,
        target: &Card,
    ) -> Result<Vec<OpcodeList>, Error> {
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
                let to = PlayerZone::new(target.owner(), Zone::Graveyard);
                let volatile = target
                    .computed()
                    .abilities
                    .contains(&KeywordAbility::Volatile);
                Ok(filter_vec![
                    if volatile {
                        None
                    } else {
                        Some(OpcodeList::new(vec![Opcode::GenerateShards {
                            player: to.player,
                            source: target.id(),
                            color: target.archetype().attribute.color,
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
                let to = PlayerZone::new(target.owner(), Zone::Hand);
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
    ) -> Result<Vec<OpcodeList>, Error> {
        let mut opcodes = vec![];
        for player in self.state.players.iter() {
            for card in player.field.iter() {
                opcodes.extend(self.apply_event(event, source, card)?);
            }
        }
        Ok(opcodes)
    }
}
