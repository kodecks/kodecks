use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "zigz",
    "Zigzag Ammonite",
    color: Color::BLUE,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 300,
);

impl Effect for CardDef {}
