use crate::{
    ability::KeywordAbility,
    card::ArchetypeId,
    color::Color,
    env::Environment,
    error::Error,
    event::{CardEvent, EventReason},
    field::FieldState,
    filter_vec,
    id::{CardId, ObjectId, TimedObjectId},
    opcode::{Opcode, OpcodeList},
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum ActionCommand {
    InflictDamage {
        target: u8,
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
        source: ObjectId,
        color: Color,
        amount: u32,
    },
    ConsumeShards {
        player: u8,
        source: ObjectId,
        color: Color,
        amount: u32,
    },
    BreakShield {
        target: TimedObjectId,
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
                let piercing = source
                    .computed()
                    .abilities
                    .contains(&KeywordAbility::Piercing);
                let shields = current_target
                    .computed()
                    .shields
                    .map(|shields| shields.value())
                    .unwrap_or_default();
                if !piercing && shields > 0 {
                    return Ok(vec![OpcodeList::new(vec![Opcode::BreakShield {
                        card: current_target.id(),
                    }])]);
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
                    return Err(Error::TargetLost { target });
                }
                env.apply_event(CardEvent::ReturnedToHand { reason }, source, current_target)
            }
            ActionCommand::ShuffleCardIntoDeck { source, target } => {
                let source = env.state.find_card(source)?;
                let current_target = env.state.find_card(target.id)?;
                if current_target.timed_id() != target {
                    return Err(Error::TargetLost { target });
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
                    return Err(Error::TargetLost { target });
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
            ActionCommand::BreakShield { target } => {
                let current_target = env.state.find_card(target.id)?;
                if current_target.timed_id() != target {
                    return Err(Error::TargetLost { target });
                }
                Ok(vec![OpcodeList::new(vec![Opcode::BreakShield {
                    card: current_target.id(),
                }])])
            }
        }
    }
}
