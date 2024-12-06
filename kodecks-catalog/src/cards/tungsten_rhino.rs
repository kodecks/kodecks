use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "tung", // spellchecker:disable-line
    "Tungsten Rhino",
    color: Color::RED,
    cost: 5,
    card_type: CardType::Creature,
    creature_type: CreatureType::Robot,
    power: 5,
    abilities: &[][..],
);

impl Effect for CardDef {}
