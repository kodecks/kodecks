use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "radi",
    "Radio Deer",
    color: Color::GREEN,
    cost: 2,
    card_type: CardType::Creature,
    power: 200,
);

impl Effect for CardDef {}
