use crate::{
    color::Color,
    env::Environment,
    error::Error,
    event::{CardEvent, EventReason},
    id::ObjectId,
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
        target: ObjectId,
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
                let target = env.state.find_card(target)?;
                env.apply_event(CardEvent::Destroyed { reason }, source, target)
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
