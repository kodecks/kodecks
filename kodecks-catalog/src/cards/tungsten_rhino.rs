use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "tung",
    "Tungsten Rhino",
    color: Color::RED,
    cost: 4,
    card_type: CardType::Creature,
    creature_type: CreatureType::Robot,
    power: 500,
);

impl Effect for CardDef {}
