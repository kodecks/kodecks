use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "oill",
    "Oil-Leaking Droid",
    color: Color::RED,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Robot,
    power: 100,
    abilities: &[KeywordAbility::Toxic][..],
    shards: 2,
);

impl Effect for CardDef {}
