use crate::scene::translator::Translator;
use fluent_bundle::FluentArgs;
use fluent_content::Request;
use kodecks::{
    card::Catalog,
    env::LocalEnvironment,
    log::LogAction,
    zone::{MoveReason, Zone},
};
use std::borrow::Cow;

pub fn translate_log<'a>(
    action: &LogAction,
    env: &LocalEnvironment,
    catalog: &Catalog,
    translator: &Translator,
) -> Option<Cow<'a, str>> {
    let mut args = FluentArgs::new();
    let id = match action {
        LogAction::GameStarted => "log-game-started",
        LogAction::GameEnded { winner, .. } => {
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
        LogAction::TurnChanged { player, turn } => {
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
        LogAction::PhaseChanged { phase } => {
            args.set("phase", phase.to_string().to_ascii_lowercase());
            "log-phase-changed"
        }
        LogAction::LifeChanged { player, life } => {
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
        LogAction::DamageTaken { player, amount } => {
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
        LogAction::DeckShuffled { player } => {
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
        LogAction::EffectActivated { source, .. } => {
            if let Ok(card) = env.find_card(*source) {
                let source = translator
                    .get(&format!("card-{}", catalog[card.archetype_id].safe_name))
                    .to_string();
                args.set("source", source);
            }
            "log-effect-activated"
        }
        LogAction::CardMoved {
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
        LogAction::CardTargeted { source, target } => {
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
        LogAction::ShardsEarned {
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
        LogAction::ShardsSpent {
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
        LogAction::CardTokenGenerated { card } => {
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
        LogAction::CardTokenDestroyed { card } => {
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
        LogAction::AttackDeclared { attacker } => {
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
        LogAction::CreatureAttackedCreature { attacker, blocker } => {
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
        LogAction::CreatureAttackedPlayer { attacker, player } => {
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
