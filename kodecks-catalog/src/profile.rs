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

    let deck_list_jade = DeckList::parse(
        "
    Vigilant Lynx 3
    Leaf-Veined Gecko 4
    Scrapyard Raven 3
    Radio Deer 2
    Moss-Grown Mastodon 2
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
