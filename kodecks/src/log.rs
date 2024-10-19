use crate::{
    color::Color, effect::EffectId, env::EndgameReason, id::ObjectId, phase::Phase,
    player::PlayerZone, zone::MoveReason,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum LogAction {
    #[strum(to_string = "Game started")]
    GameStarted,
    #[strum(to_string = "Game ended with {reason}")]
    GameEnded {
        winner: Option<u8>,
        reason: EndgameReason,
    },
    #[strum(to_string = "Turn {turn} started for {player}")]
    TurnChanged { turn: u32, player: u8 },
    #[strum(to_string = "Phase changed to {phase}")]
    PhaseChanged { phase: Phase },
    #[strum(to_string = "Attack declared by {attacker}")]
    AttackDeclared { attacker: ObjectId },
    #[strum(to_string = "{attacker} attacks {blocker}")]
    CreatureAttackedCreature {
        attacker: ObjectId,
        blocker: ObjectId,
    },
    #[strum(to_string = "{attacker} attacks {player}")]
    CreatureAttackedPlayer { attacker: ObjectId, player: u8 },
    #[strum(to_string = "Life changed for {player}: {life}")]
    LifeChanged { player: u8, life: u32 },
    #[strum(to_string = "Damage {amount} inflicted to {player}")]
    DamageTaken { player: u8, amount: u32 },
    #[strum(to_string = "Shards generated to {player} in {color} color: {amount}")]
    ShardsEarned {
        player: u8,
        source: ObjectId,
        color: Color,
        amount: u32,
    },
    #[strum(to_string = "Shards removed from {player} in {color} color: {amount}")]
    ShardsSpent {
        player: u8,
        source: ObjectId,
        color: Color,
        amount: u32,
    },
    #[strum(to_string = "Card {card} moved from {from:?} to {to:?} due to {reason:?}")]
    CardMoved {
        player: u8,
        card: ObjectId,
        from: PlayerZone,
        to: PlayerZone,
        reason: MoveReason,
    },
    #[strum(to_string = "Card Token {card} generated")]
    CardTokenGenerated { card: ObjectId },
    #[strum(to_string = "Card Token {card} destroyed")]
    CardTokenDestroyed { card: ObjectId },
    #[strum(to_string = "Deck shuffled for {player}")]
    DeckShuffled { player: u8 },
    #[strum(to_string = "Effect {id} triggered by {source}")]
    EffectActivated { source: ObjectId, id: EffectId },
    #[strum(to_string = "Card {target} targeted by {source}")]
    CardTargeted { source: ObjectId, target: ObjectId },
    #[strum(to_string = "Shield broken for {card}")]
    ShieldBroken { card: ObjectId },
}
