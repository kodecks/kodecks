use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "sola",
    "Solar Beetle",
    color: Color::RED,
    cost: 1,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 2,
    shards: 1,
);

impl Effect for CardDef {}
