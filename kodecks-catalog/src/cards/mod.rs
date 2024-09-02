use kodecks::prelude::*;

mod airborne_eagle_ray;
mod bambooster;
mod deep_sea_wyrm;
mod diamond_porcupine;
mod flash_bang_jellyfish;
mod leaf_veined_gecko;
mod mire_alligator;
mod moss_grown_mastodon;
mod oil_leaking_droid;
mod pyrosnail;
mod radio_deer;
mod scrapyard_raven;
mod turbofish;
mod vigilant_lynx;
mod volcanic_wyrm;
mod wind_up_spider;

pub static CARDS: CardMap = phf::phf_map! {
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
    "leaf-veined-gecko" => leaf_veined_gecko::ARCHETYPE,
    "leaf" => leaf_veined_gecko::ARCHETYPE,
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
};
