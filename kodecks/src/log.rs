use crate::{
    color::Color, effect::EffectId, id::ObjectId, phase::Phase, player::PlayerZone, target::Target,
    zone::MoveReason,
};
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(rename_all = "snake_case")]
pub enum LogAction {
    #[strum(to_string = "Game started")]
    GameStarted,
    #[strum(to_string = "Turn {turn} started for {player} in {phase} phase")]
    TurnChanged { turn: u32, player: u8, phase: Phase },
    #[strum(to_string = "Phase changed to {phase}")]
    PhaseChanged { phase: Phase },
    #[strum(to_string = "{attacker} attacks {target:?}")]
    Attacked { attacker: ObjectId, target: Target },
    #[strum(to_string = "Life changed for {player}: {life}")]
    LifeChanged { player: u8, life: u32 },
    #[strum(to_string = "Damage {damage} inflicted to {player}")]
    DamageInflicted { player: u8, damage: u32 },
    #[strum(to_string = "Shards generated to {player} in {color} color: {amount}")]
    ShardsGenerated {
        player: u8,
        source: ObjectId,
        color: Color,
        amount: u32,
    },
    #[strum(to_string = "Shards removed from {player} in {color} color: {amount}")]
    ShardsConsumed {
        player: u8,
        source: ObjectId,
        color: Color,
        amount: u32,
    },
    #[strum(to_string = "Card {card} moved from {from:?} to {to:?} due to {reason:?}")]
    CardMoved {
        card: ObjectId,
        from: PlayerZone,
        to: PlayerZone,
        reason: MoveReason,
    },
    #[strum(to_string = "Deck shuffled for {player}")]
    DeckShuffled { player: u8 },
    #[strum(to_string = "Effect {id} triggered by {source}")]
    EffectTriggered { source: ObjectId, id: EffectId },
    #[strum(to_string = "Card {targets:?} targeted by {source}")]
    CardsTargeted {
        source: ObjectId,
        targets: Vec<ObjectId>,
    },
}
