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
            if let Ok(card) = env.find_card(*source) {
                let source = translator
                    .get(&format!("card-{}", catalog[card.archetype_id].safe_name))
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
            if let Some(card) = env
                .find_card(*card)
                .ok()
                .filter(|card| !card.archetype_id.is_empty())
            {
                let card = translator
                    .get(&format!("card-{}", catalog[card.archetype_id].safe_name))
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
            if let Ok(card) = env.find_card(*source) {
                let source = translator
                    .get(&format!("card-{}", catalog[card.archetype_id].safe_name))
                    .to_string();
                args.set("source", source);
            } else {
                args.set("source", "unknown");
            }
            if let Ok(card) = env.find_card(*target) {
                let target = translator
                    .get(&format!("card-{}", catalog[card.archetype_id].safe_name))
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
            if let Ok(card) = env.find_card(*card) {
                let card = translator
                    .get(&format!("card-{}", catalog[card.archetype_id].safe_name))
                    .to_string();
                args.set("card", card);
            } else {
                args.set("card", "unknown");
            }
            "log-card-token-generated"
        }
        GameLog::CardTokenDestroyed { card } => {
            if let Ok(card) = env.find_card(*card) {
                let card = translator
                    .get(&format!("card-{}", catalog[card.archetype_id].safe_name))
                    .to_string();
                args.set("card", card);
            } else {
                args.set("card", "unknown");
            }
            "log-card-token-destroyed"
        }
        GameLog::AttackDeclared { attacker } => {
            if let Ok(attacker) = env.find_card(*attacker) {
                let attacker = translator
                    .get(&format!(
                        "card-{}",
                        catalog[attacker.archetype_id].safe_name
                    ))
                    .to_string();
                args.set("attacker", attacker);
            } else {
                args.set("attacker", "unknown");
            }
            "log-attack-declared"
        }
        GameLog::CreatureAttackedCreature { attacker, blocker } => {
            if let Ok(attacker) = env.find_card(*attacker) {
                let attacker = translator
                    .get(&format!(
                        "card-{}",
                        catalog[attacker.archetype_id].safe_name
                    ))
                    .to_string();
                args.set("attacker", attacker);
            } else {
                args.set("attacker", "unknown");
            }
            if let Ok(blocker) = env.find_card(*blocker) {
                let blocker = translator
                    .get(&format!("card-{}", catalog[blocker.archetype_id].safe_name))
                    .to_string();
                args.set("blocker", blocker);
            } else {
                args.set("blocker", "unknown");
            }
            "log-creature-attacked-creature"
        }
        GameLog::CreatureAttackedPlayer { attacker, player } => {
            if let Ok(attacker) = env.find_card(*attacker) {
                let attacker = translator
                    .get(&format!(
                        "card-{}",
                        catalog[attacker.archetype_id].safe_name
                    ))
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
