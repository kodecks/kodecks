use crate::{
    card::CardSnapshot, color::Color, effect::EffectId, env::EndgameReason, phase::Phase,
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
        turn: u16,
        player: u8,
    },
    PhaseChanged {
        phase: Phase,
    },
    AttackDeclared {
        attacker: CardSnapshot,
    },
    CreatureAttackedCreature {
        attacker: CardSnapshot,
        blocker: CardSnapshot,
    },
    CreatureAttackedPlayer {
        attacker: CardSnapshot,
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
        source: CardSnapshot,
        color: Color,
        amount: u8,
    },
    ShardsSpent {
        player: u8,
        source: CardSnapshot,
        color: Color,
        amount: u8,
    },
    CardMoved {
        player: u8,
        card: CardSnapshot,
        from: PlayerZone,
        to: PlayerZone,
        reason: MoveReason,
    },
    CardTokenGenerated {
        card: CardSnapshot,
    },
    CardTokenDestroyed {
        card: CardSnapshot,
    },
    DeckShuffled {
        player: u8,
    },
    EffectActivated {
        source: CardSnapshot,
        id: EffectId,
    },
    CardTargeted {
        source: CardSnapshot,
        target: CardSnapshot,
    },
    ShieldBroken {
        card: CardSnapshot,
    },
}

impl GameLog {
    pub fn redacted(self, viewer: u8) -> Self {
        match self {
            Self::AttackDeclared { attacker } => Self::AttackDeclared {
                attacker: attacker.redacted(viewer),
            },
            Self::CreatureAttackedCreature { attacker, blocker } => {
                Self::CreatureAttackedCreature {
                    attacker: attacker.redacted(viewer),
                    blocker: blocker.redacted(viewer),
                }
            }
            Self::CreatureAttackedPlayer { attacker, player } => Self::CreatureAttackedPlayer {
                attacker: attacker.redacted(viewer),
                player,
            },
            Self::ShardsEarned {
                player,
                source,
                color,
                amount,
            } => Self::ShardsEarned {
                player,
                source: source.redacted(viewer),
                color,
                amount,
            },
            Self::ShardsSpent {
                player,
                source,
                color,
                amount,
            } => Self::ShardsSpent {
                player,
                source: source.redacted(viewer),
                color,
                amount,
            },
            Self::CardMoved {
                player,
                card,
                from,
                to,
                reason,
            } => Self::CardMoved {
                player,
                card: card.redacted(viewer),
                from,
                to,
                reason,
            },
            Self::CardTokenGenerated { card } => Self::CardTokenGenerated {
                card: card.redacted(viewer),
            },
            Self::CardTokenDestroyed { card } => Self::CardTokenDestroyed {
                card: card.redacted(viewer),
            },
            Self::EffectActivated { source, id } => Self::EffectActivated {
                source: source.redacted(viewer),
                id,
            },
            Self::CardTargeted { source, target } => Self::CardTargeted {
                source: source.redacted(viewer),
                target: target.redacted(viewer),
            },
            Self::ShieldBroken { card } => Self::ShieldBroken {
                card: card.redacted(viewer),
            },
            _ => self,
        }
    }
}
