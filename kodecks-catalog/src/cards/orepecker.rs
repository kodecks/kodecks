use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "orep",
    "Orepecker",
    color: Color::RED,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Robot,
    power: 1,
    abilities: &[][..],
);

impl Effect for CardDef {}
