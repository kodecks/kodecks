use crate::{
    card::Card,
    color::Color,
    event::CardEvent,
    field::{FieldBattleState, FieldState},
    id::ObjectId,
    phase::Phase,
    player::PlayerZone,
    target::Target,
    zone::MoveReason,
};
use std::fmt;

#[derive(Debug, Clone)]
pub enum Opcode {
    StartGame,
    ChangeTurn {
        turn: u32,
        player: u8,
        phase: Phase,
    },
    ChangePhase {
        phase: Phase,
    },
    SetLife {
        player: u8,
        life: u32,
    },
    ReduceCost {
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
        card: ObjectId,
    },
    GenerateCardToken {
        card: Card,
    },
    DrawCard {
        player: u8,
    },
    CastCard {
        player: u8,
        card: ObjectId,
        cost: u32,
    },
    MoveCard {
        card: ObjectId,
        from: PlayerZone,
        to: PlayerZone,
        reason: MoveReason,
    },
    ShuffleDeck {
        player: u8,
    },
    TriggerEvent {
        source: ObjectId,
        target: ObjectId,
        event: CardEvent,
    },
    SetFieldState {
        card: ObjectId,
        state: FieldState,
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
        player: u8,
        amount: u32,
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

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
