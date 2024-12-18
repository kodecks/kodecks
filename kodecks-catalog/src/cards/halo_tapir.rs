use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "halo",
    "Halo Tapir",
    color: Color::GREEN,
    cost: 3,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 4,
    shards: 1,
);

impl Effect for CardDef {}
