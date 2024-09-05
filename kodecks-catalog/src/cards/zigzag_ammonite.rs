use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "zigz",
    "Zigzag Ammonite",
    color: Color::AZURE,
    cost: 2,
    card_type: CardType::Creature,
    power: 300,
);

impl Effect for CardDef {}
