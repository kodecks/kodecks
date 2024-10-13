use kodecks::prelude::*;

// card-modules
mod airborne_eagle_ray;
mod amalgam_rat;
mod ant;
mod badlands_chimera;
mod bambooster;
mod binary_starfish;
mod coppermine_scorpion;
mod deep_sea_wyrm;
mod demilune_nighthawk;
mod diamond_porcupine;
mod electric_clione;
mod electric_flytrap;
mod evergreen_flamingo;
mod flash_bang_jellyfish;
mod graphite_armadillo;
mod halo_tapir;
mod helium_puffer;
mod icefall_weasel;
mod laser_frog;
mod marshland_moose;
mod minimum_bear;
mod mire_alligator;
mod moonlit_gecko;
mod moss_grown_mastodon;
mod oil_leaking_droid;
mod orepecker;
mod poison_tusk_babirusa;
mod pyrosnail;
mod quartz_moth;
mod quicksand_skulker;
mod radio_deer;
mod saltmarsh_moray;
mod scrapyard_raven;
mod solar_beetle;
mod soundless_owl;
mod subspace_mole;
mod thermite_crab;
mod tungsten_rhino;
mod turbofish;
mod vigilant_lynx;
mod volcanic_wyrm;
mod voracious_anteater;
mod wasteland_cobra;
mod wind_up_spider;
mod wiretap_vine;
mod zigzag_ammonite;

pub static CARDS: CardMap = phf::phf_map! {
    // card-entries
    "badlands-chimera" => badlands_chimera::ARCHETYPE,
    "badl" => badlands_chimera::ARCHETYPE,
    "halo-tapir" => halo_tapir::ARCHETYPE,
    "halo" => halo_tapir::ARCHETYPE,
    "vigilant-lynx" => vigilant_lynx::ARCHETYPE,
    "vigi" => vigilant_lynx::ARCHETYPE,
    "bambooster" => bambooster::ARCHETYPE,
    "bamb" => bambooster::ARCHETYPE,
    "scrapyard-raven" => scrapyard_raven::ARCHETYPE,
    "scra" => scrapyard_raven::ARCHETYPE,
    "wind-up-spider" => wind_up_spider::ARCHETYPE,
    "wind" => wind_up_spider::ARCHETYPE,
    "mire-alligator" => mire_alligator::ARCHETYPE,
    "mire" => mire_alligator::ARCHETYPE,
    "moss-grown-mastodon" => moss_grown_mastodon::ARCHETYPE,
    "moss" => moss_grown_mastodon::ARCHETYPE,
    "radio-deer" => radio_deer::ARCHETYPE,
    "radi" => radio_deer::ARCHETYPE,
    "moonlit-gecko" => moonlit_gecko::ARCHETYPE,
    "moon" => moonlit_gecko::ARCHETYPE,
    "diamond-porcupine" => diamond_porcupine::ARCHETYPE,
    "diam" => diamond_porcupine::ARCHETYPE,
    "pyrosnail" => pyrosnail::ARCHETYPE,
    "pyro" => pyrosnail::ARCHETYPE,
    "oil-leaking-droid" => oil_leaking_droid::ARCHETYPE,
    "oill" => oil_leaking_droid::ARCHETYPE,
    "volcanic-wyrm" => volcanic_wyrm::ARCHETYPE,
    "volc" => volcanic_wyrm::ARCHETYPE,
    "turbofish" => turbofish::ARCHETYPE,
    "turb" => turbofish::ARCHETYPE,
    "flash-bang-jellyfish" => flash_bang_jellyfish::ARCHETYPE,
    "flas" => flash_bang_jellyfish::ARCHETYPE,
    "airborne-eagle-ray" => airborne_eagle_ray::ARCHETYPE,
    "airb" => airborne_eagle_ray::ARCHETYPE,
    "deep-sea-wyrm" => deep_sea_wyrm::ARCHETYPE,
    "deep" => deep_sea_wyrm::ARCHETYPE,
    "wasteland-cobra" => wasteland_cobra::ARCHETYPE,
    "wast" => wasteland_cobra::ARCHETYPE,
    "coppermine-scorpion" => coppermine_scorpion::ARCHETYPE,
    "copp" => coppermine_scorpion::ARCHETYPE,
    "zigzag-ammonite" => zigzag_ammonite::ARCHETYPE,
    "zigz" => zigzag_ammonite::ARCHETYPE,
    "voracious-anteater" => voracious_anteater::ARCHETYPE,
    "vora" => voracious_anteater::ARCHETYPE,
    "ant" => ant::ARCHETYPE,
    "binary-starfish" => binary_starfish::ARCHETYPE,
    "bina" => binary_starfish::ARCHETYPE,
    "laser-frog" => laser_frog::ARCHETYPE,
    "lase" => laser_frog::ARCHETYPE,
    "helium-puffer" => helium_puffer::ARCHETYPE,
    "heli" => helium_puffer::ARCHETYPE,
    "graphite-armadillo" => graphite_armadillo::ARCHETYPE,
    "grap" => graphite_armadillo::ARCHETYPE,
    "demilune-nighthawk" => demilune_nighthawk::ARCHETYPE,
    "demi" => demilune_nighthawk::ARCHETYPE,
    "icefall-weasel" => icefall_weasel::ARCHETYPE,
    "icef" => icefall_weasel::ARCHETYPE,
    "marshland-moose" => marshland_moose::ARCHETYPE,
    "mars" => marshland_moose::ARCHETYPE,
    "quartz-moth" => quartz_moth::ARCHETYPE,
    "quar" => quartz_moth::ARCHETYPE,
    "electric-clione" => electric_clione::ARCHETYPE,
    "elec" => electric_clione::ARCHETYPE,
    "electric-flytrap" => electric_flytrap::ARCHETYPE,
    "elfl" => electric_flytrap::ARCHETYPE,
    "tungsten-rhino" => tungsten_rhino::ARCHETYPE,
    "tung" => tungsten_rhino::ARCHETYPE,
    "saltmarsh-moray" => saltmarsh_moray::ARCHETYPE,
    "salt" => saltmarsh_moray::ARCHETYPE,
    "poison-tusk-babirusa" => poison_tusk_babirusa::ARCHETYPE,
    "pois" => poison_tusk_babirusa::ARCHETYPE,
    "minimum-bear" => minimum_bear::ARCHETYPE,
    "mini" => minimum_bear::ARCHETYPE,
    "solar-beetle" => solar_beetle::ARCHETYPE,
    "sola" => solar_beetle::ARCHETYPE,
    "soundless-owl" => soundless_owl::ARCHETYPE,
    "soun" => soundless_owl::ARCHETYPE,
    "quicksand-skulker" => quicksand_skulker::ARCHETYPE,
    "quic" => quicksand_skulker::ARCHETYPE,
    "wiretap-vine" => wiretap_vine::ARCHETYPE,
    "wire" => wiretap_vine::ARCHETYPE,
    "orepecker" => orepecker::ARCHETYPE,
    "orep" => orepecker::ARCHETYPE,
    "evergreen-flamingo" => evergreen_flamingo::ARCHETYPE,
    "ever" => evergreen_flamingo::ARCHETYPE,
    "thermite-crab" => thermite_crab::ARCHETYPE,
    "ther" => thermite_crab::ARCHETYPE,
    "amalgam-rat" => amalgam_rat::ARCHETYPE,
    "amal" => amalgam_rat::ARCHETYPE,
    "subspace-mole" => subspace_mole::ARCHETYPE,
    "subs" => subspace_mole::ARCHETYPE,
};
