use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "wast",
    "Wasteland Cobra",
    color: Color::GREEN,
    cost: 1,
    card_type: CardType::Creature,
    power: 200,
);

impl Effect for CardDef {}
