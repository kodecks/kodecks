use crate::{
    card::Card,
    color::Color,
    event::CardEvent,
    field::{FieldBattleState, FieldState},
    id::ObjectId,
    phase::Phase,
    player::Zone,
    target::Target,
    zone::MoveReason,
};
use std::fmt;

#[derive(Debug, Clone)]
pub enum Opcode {
    StartGame,
    ChangeTurn {
        turn: u16,
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
    ConsumeManas {
        player: u8,
        amount: u8,
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
        cost: u8,
    },
    MoveCard {
        card: ObjectId,
        from: Zone,
        to: Zone,
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
    ResetPlayerState {
        player: u8,
    },
    InflictDamage {
        player: u8,
        amount: u8,
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
