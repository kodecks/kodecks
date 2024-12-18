use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "quar",
    "Quartz Moth",
    color: Color::RED,
    cost: 0,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 100,
    shards: 1,
);

impl Effect for CardDef {}
