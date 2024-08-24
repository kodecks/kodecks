use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "radi",
    "Radio Deer",
    color: Color::JADE,
    cost: 3,
    card_type: CardType::Creature,
    power: 300,
);

impl Effect for CardDef {}
