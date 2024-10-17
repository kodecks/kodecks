use fluent_bundle::FluentArgs;
use fluent_content::Request;
use kodecks::{card::Catalog, env::LocalEnvironment, log::LogAction};

pub fn get_request<'a>(
    action: &LogAction,
    env: &LocalEnvironment,
    catalog: &Catalog,
) -> Option<Request<'a, FluentArgs<'a>>> {
    let mut args = FluentArgs::new();
    let id = match action {
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
                args.set("source", catalog[card.archetype_id].name);
            }
            "log-effect-activated"
        }
        _ => return None,
    };
    Some(Request {
        id,
        attr: None,
        args: Some(args),
    })
}
