use std::borrow::Cow;

use crate::scene::translator::Translator;
use fluent_bundle::FluentArgs;
use fluent_content::Request;
use kodecks::{card::Catalog, env::LocalEnvironment, log::LogAction};

pub fn translate_log<'a>(
    action: &LogAction,
    env: &LocalEnvironment,
    catalog: &Catalog,
    translator: &Translator,
) -> Option<Cow<'a, str>> {
    let mut args = FluentArgs::new();
    let id = match action {
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
        _ => return None,
    };
    Some(translator.get(Request {
        id,
        attr: None,
        args: Some(args),
    }))
}
