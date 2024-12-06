use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "wire",
    "Wiretap Vine",
    color: Color::BLUE,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Program,
    power: 2,
);

impl Effect for CardDef {}
