use crate::CATALOG;
use kodecks::{
    config::{DebugFlags, GameConfig},
    deck::DeckList,
    player::PlayerConfig,
    profile::{BotConfig, GameProfile},
};

pub fn default_profile() -> GameProfile {
    let deck_list_red = DeckList::parse(
        "
    Volcanic Wyrm 2
    Wind-Up Spider 3
    Pyrosnail 3
    Oil-Leaking Droid 3
    Diamond Porcupine 3
    Bambooster 3
    Coppermine Scorpion 3
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
        config: GameConfig {
            debug: DebugFlags::DEBUG_COMMAND,
            initial_life: 1000,
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
        scenario: None,
    }
}
