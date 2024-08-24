use std::fmt;

use crate::{
    color::Color,
    event::CardEvent,
    field::{FieldBattleState, FieldCardState},
    id::ObjectId,
    phase::Phase,
    player::{PlayerId, PlayerZone},
    target::Target,
    zone::MoveReason,
};

#[derive(Debug, Clone)]
pub enum Opcode {
    StartGame,
    ChangeTurn {
        turn: u32,
        player: PlayerId,
        phase: Phase,
    },
    ChangePhase {
        phase: Phase,
    },
    SetLife {
        player: PlayerId,
        life: u32,
    },
    ReduceCost {
        player: PlayerId,
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
    DrawCard {
        player: PlayerId,
    },
    CastCard {
        player: PlayerId,
        card: ObjectId,
    },
    MoveCard {
        card: ObjectId,
        from: PlayerZone,
        to: PlayerZone,
        reason: MoveReason,
    },
    TriggerEvent {
        source: ObjectId,
        target: ObjectId,
        event: CardEvent,
    },
    SetFieldCardState {
        card: ObjectId,
        state: FieldCardState,
    },
    Attack {
        attacker: ObjectId,
        target: Target,
    },
    SetBattleState {
        card: ObjectId,
        state: Option<FieldBattleState>,
    },
    ResetBattleState,
    InflictDamage {
        player: PlayerId,
        damage: u32,
    },
}

#[derive(Debug, Clone)]
pub struct OpcodeList(Vec<Opcode>);

impl OpcodeList {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Opcode>,
    {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for OpcodeList {
    type Item = Opcode;
    type IntoIter = std::vec::IntoIter<Opcode>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl fmt::Display for OpcodeList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
