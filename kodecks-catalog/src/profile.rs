use crate::CATALOG;
use kodecks::{
    deck::DeckList,
    player::PlayerConfig,
    profile::{BotConfig, DebugConfig, DebugFlags, GameProfile},
    regulation::Regulation,
};

pub fn default_profile() -> GameProfile {
    let deck_list_red = DeckList::parse(
        "
    Volcanic Wyrm 2
    Wind-Up Spider 2
    Pyrosnail 2
    Oil-Leaking Droid 2
    Diamond Porcupine 2
    Bambooster 2
    Coppermine Scorpion 2
    Laser Frog 3
    Graphite Armadillo 3
    ",
        &CATALOG,
    )
    .unwrap();

    let deck_list_green = DeckList::parse(
        "
    Vigilant Lynx 3
    Moonlit Gecko 4
    Scrapyard Raven 3
    Radio Deer 1
    Moss-Grown Mastodon 2
    Voracious Anteater 1
    Mire Alligator 3
    Wasteland Cobra 3
    ",
        &CATALOG,
    )
    .unwrap();

    GameProfile {
        regulation: Regulation {
            initial_life: 1000,
            ..Default::default()
        },
        debug: DebugConfig {
            flags: DebugFlags::DEBUG_COMMAND,
            ..Default::default()
        },
        players: vec![
            PlayerConfig {
                id: 1,
                deck: deck_list_green,
            },
            PlayerConfig {
                id: 2,
                deck: deck_list_red,
            },
        ],
        bots: vec![BotConfig { player: 2 }],
    }
}
