use crate::{
    archetype::ArchetypeId,
    color::Color,
    env::Environment,
    error::ActionError,
    event::{CardEvent, EventReason},
    field::FieldState,
    filter_vec,
    id::{ObjectId, TimedCardId, TimedObjectId},
    opcode::{Opcode, OpcodeList},
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, Encode, Decode)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum ActionCommand {
    InflictDamage {
        target: u8,
        amount: u8,
    },
    DestroyCard {
        source: ObjectId,
        target: TimedObjectId,
        reason: EventReason,
    },
    ReturnCardToHand {
        source: ObjectId,
        target: TimedObjectId,
        reason: EventReason,
    },
    ShuffleCardIntoDeck {
        source: ObjectId,
        target: TimedObjectId,
    },
    SetFieldState {
        source: ObjectId,
        target: TimedObjectId,
        state: FieldState,
        reason: EventReason,
    },
    GenerateCardToken {
        token: ObjectId,
        archetype: ArchetypeId,
        player: u8,
    },
    GenerateShards {
        player: u8,
        color: Color,
        amount: u8,
    },
    ConsumeShards {
        player: u8,
        color: Color,
        amount: u8,
    },
}

impl ActionCommand {
    pub fn into_opcodes(self, env: &Environment) -> Result<Vec<OpcodeList>, ActionError> {
        match self {
            ActionCommand::InflictDamage { target, amount } => {
                Ok(vec![OpcodeList::new(vec![Opcode::InflictDamage {
                    player: target,
                    amount,
                }])])
            }
            ActionCommand::DestroyCard {
                source,
                target,
                reason,
            } => {
                let source = env.state.find_card(source)?;
                let current_target = env.state.find_card(target.id)?;
                if current_target.timed_id() != target {
                    return Err(ActionError::TargetLost { target });
                }
                let from = *current_target.zone();
                env.apply_event(
                    CardEvent::Destroyed { from, reason },
                    source,
                    current_target,
                )
            }
            ActionCommand::ReturnCardToHand {
                source,
                target,
                reason,
            } => {
                let source = env.state.find_card(source)?;
                let current_target = env.state.find_card(target.id)?;
                if current_target.timed_id() != target {
                    return Err(ActionError::TargetLost { target });
                }
                env.apply_event(CardEvent::ReturnedToHand { reason }, source, current_target)
            }
            ActionCommand::ShuffleCardIntoDeck { source, target } => {
                let source = env.state.find_card(source)?;
                let current_target = env.state.find_card(target.id)?;
                if current_target.timed_id() != target {
                    return Err(ActionError::TargetLost { target });
                }
                Ok(filter_vec![
                    env.apply_event(CardEvent::ReturnedToDeck, source, current_target)?,
                    Some(OpcodeList::new(vec![Opcode::ShuffleDeck {
                        player: current_target.owner(),
                    }])),
                ])
            }
            ActionCommand::SetFieldState { target, state, .. } => {
                let current_target = env.state.find_card(target.id)?;
                if current_target.timed_id() != target {
                    return Err(ActionError::TargetLost { target });
                }
                Ok(vec![OpcodeList::new(vec![Opcode::SetFieldState {
                    card: current_target.id(),
                    state,
                }])])
            }
            ActionCommand::GenerateCardToken {
                token,
                archetype,
                player,
            } => {
                let card = env.generate_card_token(player, token, archetype);
                let from = *card.zone();
                let casted = env
                    .apply_event(CardEvent::Casted { from }, &card, &card)
                    .ok()
                    .into_iter()
                    .flatten();
                let casted_any = env
                    .apply_event_any(CardEvent::AnyCasted, &card)
                    .ok()
                    .into_iter()
                    .flatten();
                Ok(filter_vec![
                    Some(OpcodeList::new(vec![Opcode::GenerateCardToken { card }])),
                    casted,
                    casted_any,
                ])
            }
            ActionCommand::GenerateShards {
                player,
                color,
                amount,
            } => Ok(vec![OpcodeList::new(vec![Opcode::GenerateShards {
                player,
                color,
                amount,
            }])]),
            ActionCommand::ConsumeShards {
                player,
                color,
                amount,
            } => Ok(vec![OpcodeList::new(vec![Opcode::ConsumeShards {
                player,
                color,
                amount,
            }])]),
        }
    }
}
