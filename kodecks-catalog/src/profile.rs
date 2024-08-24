use crate::CATALOG;
use kodecks::{
    config::{DebugFlags, GameConfig},
    deck::DeckList,
    player::{PlayerConfig, PlayerId},
    profile::GameProfile,
};

pub fn default_profile() -> GameProfile {
    let deck_list_ruby = DeckList::parse(
        "
    Volcanic Wyrm 2
    Wind-Up Spider 4
    Pyrosnail 4
    Oil-Leaking Droid 4
    Diamond Porcupine 3
    Bambooster 3
    ",
        &CATALOG,
    )
    .unwrap();

    let deck_list_jade = DeckList::parse(
        "
    Vigilant Lynx 4
    Leaf-Veined Gecko 4
    Scrapyard Raven 3
    Radio Deer 4
    Moss-Grown Mastodon 2
    Mire Alligator 3
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
                id: PlayerId::new("player1"),
                deck: deck_list_jade,
            },
            PlayerConfig {
                id: PlayerId::new("player2"),
                deck: deck_list_ruby,
            },
        ],
        scenario: None,
    }
}
