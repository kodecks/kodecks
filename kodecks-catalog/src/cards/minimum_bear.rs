use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "mini",
    "Minimum Bear",
    color: Color::BLUE,
    cost: 1,
    card_type: CardType::Creature,
    creature_type: CreatureType::Program,
    power: 1,
);

impl Effect for CardDef {}
