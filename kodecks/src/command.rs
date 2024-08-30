use crate::{
    color::Color,
    env::Environment,
    error::Error,
    event::{CardEvent, EventReason},
    field::FieldState,
    id::{CardId, ObjectId, TimedObjectId},
    opcode::{Opcode, OpcodeList},
    player::PlayerId,
};
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum ActionCommand {
    InflictDamage {
        target: PlayerId,
        damage: u32,
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
    SetFieldState {
        source: ObjectId,
        target: TimedObjectId,
        state: FieldState,
        reason: EventReason,
    },
    GenerateShards {
        player: PlayerId,
        source: ObjectId,
        color: Color,
        amount: u32,
    },
    ConsumeShards {
        player: PlayerId,
        source: ObjectId,
        color: Color,
        amount: u32,
    },
}

impl ActionCommand {
    pub fn into_opcodes(self, env: &Environment) -> Result<Vec<OpcodeList>, Error> {
        match self {
            ActionCommand::InflictDamage { target, damage } => {
                Ok(vec![OpcodeList::new(vec![Opcode::InflictDamage {
                    player: target,
                    damage,
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
                    return Err(Error::TargetLost { target });
                }
                env.apply_event(CardEvent::Destroyed { reason }, source, current_target)
            }
            ActionCommand::ReturnCardToHand {
                source,
                target,
                reason,
            } => {
                let source = env.state.find_card(source)?;
                let current_target = env.state.find_card(target.id)?;
                if current_target.timed_id() != target {
                    return Err(Error::TargetLost { target });
                }
                env.apply_event(CardEvent::ReturnedToHand { reason }, source, current_target)
            }
            ActionCommand::SetFieldState { target, state, .. } => {
                let current_target = env.state.find_card(target.id)?;
                if current_target.timed_id() != target {
                    return Err(Error::TargetLost { target });
                }
                Ok(vec![OpcodeList::new(vec![Opcode::SetFieldState {
                    card: current_target.id(),
                    state,
                }])])
            }
            ActionCommand::GenerateShards {
                player,
                source,
                color,
                amount,
            } => Ok(vec![OpcodeList::new(vec![Opcode::GenerateShards {
                player,
                source,
                color,
                amount,
            }])]),
            ActionCommand::ConsumeShards {
                player,
                source,
                color,
                amount,
            } => Ok(vec![OpcodeList::new(vec![Opcode::ConsumeShards {
                player,
                source,
                color,
                amount,
            }])]),
        }
    }
}
