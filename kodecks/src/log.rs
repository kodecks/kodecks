use crate::{
    color::Color, effect::EffectId, env::EndgameReason, id::ObjectId, phase::Phase,
    player::PlayerZone, zone::MoveReason,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum GameLog {
    GameStarted,
    GameEnded {
        winner: Option<u8>,
        reason: EndgameReason,
    },
    TurnChanged {
        turn: u32,
        player: u8,
    },
    PhaseChanged {
        phase: Phase,
    },
    AttackDeclared {
        attacker: ObjectId,
    },
    CreatureAttackedCreature {
        attacker: ObjectId,
        blocker: ObjectId,
    },
    CreatureAttackedPlayer {
        attacker: ObjectId,
        player: u8,
    },
    LifeChanged {
        player: u8,
        life: u32,
    },
    DamageTaken {
        player: u8,
        amount: u32,
    },
    ShardsEarned {
        player: u8,
        source: ObjectId,
        color: Color,
        amount: u32,
    },
    ShardsSpent {
        player: u8,
        source: ObjectId,
        color: Color,
        amount: u32,
    },
    CardMoved {
        player: u8,
        card: ObjectId,
        from: PlayerZone,
        to: PlayerZone,
        reason: MoveReason,
    },
    CardTokenGenerated {
        card: ObjectId,
    },
    CardTokenDestroyed {
        card: ObjectId,
    },
    DeckShuffled {
        player: u8,
    },
    EffectActivated {
        source: ObjectId,
        id: EffectId,
    },
    CardTargeted {
        source: ObjectId,
        target: ObjectId,
    },
    ShieldBroken {
        card: ObjectId,
    },
}
