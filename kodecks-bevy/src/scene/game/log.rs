use crate::scene::translator::Translator;
use fluent_bundle::FluentArgs;
use fluent_content::Request;
use kodecks::{
    card::Catalog,
    env::LocalEnvironment,
    log::GameLog,
    zone::{MoveReason, Zone},
};
use std::borrow::Cow;

pub fn translate_log<'a>(
    action: &GameLog,
    env: &LocalEnvironment,
    catalog: &Catalog,
    translator: &Translator,
) -> Option<Cow<'a, str>> {
    let mut args = FluentArgs::new();
    let id = match action {
        GameLog::GameStarted => "log-game-started",
        GameLog::GameEnded { winner, .. } => {
            if let Some(winner) = winner {
                args.set(
                    "winner",
                    if *winner == env.player {
                        "you"
                    } else {
                        "opponent"
                    },
                );
                "log-game-ended"
            } else {
                "log-game-ended-draw"
            }
        }
        GameLog::TurnChanged { player, turn } => {
            args.set(
                "player",
                if *player == env.player {
                    "you"
                } else {
                    "opponent"
                },
            );
            args.set("turn", turn);
            "log-turn-changed"
        }
        GameLog::PhaseChanged { phase } => {
            args.set("phase", phase.to_string().to_ascii_lowercase());
            "log-phase-changed"
        }
        GameLog::LifeChanged { player, life } => {
            args.set(
                "player",
                if *player == env.player {
                    "you"
                } else {
                    "opponent"
                },
            );
            args.set("life", life);
            "log-life-changed"
        }
        GameLog::DamageTaken { player, amount } => {
            args.set(
                "player",
                if *player == env.player {
                    "you"
                } else {
                    "opponent"
                },
            );
            args.set("amount", amount);
            "log-damage-taken"
        }
        GameLog::DeckShuffled { player } => {
            args.set(
                "player",
                if *player == env.player {
                    "you"
                } else {
                    "opponent"
                },
            );
            "log-deck-shuffled"
        }
        GameLog::EffectActivated { source, .. } => {
            if let Some(archetype) = catalog.get(source.archetype_id) {
                let source = translator
                    .get(&format!("card-{}", archetype.safe_name))
                    .to_string();
                args.set("source", source);
            }
            "log-effect-activated"
        }
        GameLog::CardMoved {
            player,
            card,
            from,
            to,
            reason,
        } => {
            args.set(
                "player",
                if *player == env.player {
                    "you"
                } else {
                    "opponent"
                },
            );
            if let Some(archetype) = catalog.get(card.archetype_id) {
                let card = translator
                    .get(&format!("card-{}", archetype.safe_name))
                    .to_string();
                args.set("card", card);
            } else {
                args.set("card", "unknown");
            }
            match reason {
                MoveReason::Draw => "log-card-drawn",
                MoveReason::Casted => "log-card-played",
                MoveReason::Destroyed if to.zone == Zone::Graveyard => {
                    "log-card-destroyed-to-graveyard"
                }
                MoveReason::Discarded => "log-card-discarded",
                _ => {
                    args.set(
                        "from-player",
                        if from.player == env.player {
                            "you"
                        } else {
                            "opponent"
                        },
                    );
                    args.set("from-zone", from.zone.to_string().to_ascii_lowercase());
                    args.set(
                        "to-player",
                        if to.player == env.player {
                            "you"
                        } else {
                            "opponent"
                        },
                    );
                    args.set("to-zone", to.zone.to_string().to_ascii_lowercase());
                    "log-card-moved"
                }
            }
        }
        GameLog::CardTargeted { source, target } => {
            if let Some(card) = catalog.get(source.archetype_id) {
                let source = translator
                    .get(&format!("card-{}", card.safe_name))
                    .to_string();
                args.set("source", source);
            } else {
                args.set("source", "unknown");
            }
            if let Some(card) = catalog.get(target.archetype_id) {
                let target = translator
                    .get(&format!("card-{}", card.safe_name))
                    .to_string();
                args.set("target", target);
            } else {
                args.set("target", "unknown");
            }
            "log-card-targeted"
        }
        GameLog::ShardsEarned {
            player,
            color,
            amount,
            ..
        } => {
            args.set(
                "player",
                if *player == env.player {
                    "you"
                } else {
                    "opponent"
                },
            );
            args.set("color", color.to_string().to_ascii_lowercase());
            args.set("amount", amount);
            "log-shards-earned"
        }
        GameLog::ShardsSpent {
            player,
            color,
            amount,
            ..
        } => {
            args.set(
                "player",
                if *player == env.player {
                    "you"
                } else {
                    "opponent"
                },
            );
            args.set("color", color.to_string().to_ascii_lowercase());
            args.set("amount", amount);
            "log-shards-spent"
        }
        GameLog::CardTokenGenerated { card } => {
            if let Some(archetype) = catalog.get(card.archetype_id) {
                let card = translator
                    .get(&format!("card-{}", archetype.safe_name))
                    .to_string();
                args.set("card", card);
            } else {
                args.set("card", "unknown");
            }
            "log-card-token-generated"
        }
        GameLog::CardTokenDestroyed { card } => {
            if let Some(archetype) = catalog.get(card.archetype_id) {
                let card = translator
                    .get(&format!("card-{}", archetype.safe_name))
                    .to_string();
                args.set("card", card);
            } else {
                args.set("card", "unknown");
            }
            "log-card-token-destroyed"
        }
        GameLog::AttackDeclared { attacker } => {
            if let Some(attacker) = catalog.get(attacker.archetype_id) {
                let attacker = translator
                    .get(&format!("card-{}", attacker.safe_name))
                    .to_string();
                args.set("attacker", attacker);
            } else {
                args.set("attacker", "unknown");
            }
            "log-attack-declared"
        }
        GameLog::CreatureAttackedCreature { attacker, blocker } => {
            if let Some(attacker) = catalog.get(attacker.archetype_id) {
                let attacker = translator
                    .get(&format!("card-{}", attacker.safe_name))
                    .to_string();
                args.set("attacker", attacker);
            } else {
                args.set("attacker", "unknown");
            }
            if let Some(blocker) = catalog.get(blocker.archetype_id) {
                let blocker = translator
                    .get(&format!("card-{}", blocker.safe_name))
                    .to_string();
                args.set("blocker", blocker);
            } else {
                args.set("blocker", "unknown");
            }
            "log-creature-attacked-creature"
        }
        GameLog::CreatureAttackedPlayer { attacker, player } => {
            if let Some(attacker) = catalog.get(attacker.archetype_id) {
                let attacker = translator
                    .get(&format!("card-{}", attacker.safe_name))
                    .to_string();
                args.set("attacker", attacker);
            } else {
                args.set("attacker", "unknown");
            }
            args.set(
                "player",
                if *player == env.player {
                    "you"
                } else {
                    "opponent"
                },
            );
            "log-creature-attacked-player"
        }
        _ => return None,
    };
    Some(translator.get(Request {
        id,
        attr: None,
        args: Some(args),
    }))
}
