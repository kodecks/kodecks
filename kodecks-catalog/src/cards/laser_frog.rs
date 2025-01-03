use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "lase",
    "Laser Frog",
    color: Color::RED,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 100,
    shards: 3,
);

impl Effect for CardDef {}
