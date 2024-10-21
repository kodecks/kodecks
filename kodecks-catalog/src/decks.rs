use crate::CATALOG;
use kodecks::deck::DeckList;

pub fn starter_deck() -> DeckList {
    DeckList::parse(
        "
    # Starter deck
    Vigilant Lynx 1
    Moonlit Gecko 1
    Scrapyard Raven 2
    Radio Deer 1
    Moss-Grown Mastodon 1
    Voracious Anteater 2
    Mire Alligator 1
    Wasteland Cobra 1
    Marshland Moose 1
    Electric Flytrap 1
    Poison-Tusk Babirusa 1
    Quicksand Skulker 1
    Evergreen Flamingo 1
    Halo Tapir 1
    Badlands Chimera 1
    Hostile Coyote 1
    Quagmire Trilobite 1
    Cenote Otter 1
    ",
        &CATALOG,
    )
    .unwrap()
}

pub fn blue_deck() -> DeckList {
    DeckList::parse(
        "
    # Blue deck
    Deep-Sea Wyrm 2
    Airborne Eagle Ray 1
    Binary Starfish 1
    Demilune Nighthawk 1
    Electric Clione 2
    Flash-Bang Jellyfish 1
    Helium Puffer 1
    Icefall Weasel 1
    Turbofish 2
    Saltmarsh Moray 1
    Minimum Bear 1
    Soundless Owl 2
    Wiretap Vine 1
    Subspace Mole 1
    Auto Parrot 1
    Awkward Auk 1
    ",
        &CATALOG,
    )
    .unwrap()
}

pub fn red_deck() -> DeckList {
    DeckList::parse(
        "
    # Red deck
    Volcanic Wyrm 2
    Wind-Up Spider 1
    Pyrosnail 1
    Oil-Leaking Droid 2
    Diamond Porcupine 1
    Bambooster 1
    Coppermine Scorpion 1
    Laser Frog 1
    Graphite Armadillo 1
    Tungsten Rhino 2
    Solar Beetle 1
    Orepecker 1
    Thermite Crab 1
    Amalgam Rat 1
    Amethyst Mantis 1
    Cyanide Spitter 1
    Ruby Digger 1
    ",
        &CATALOG,
    )
    .unwrap()
}
