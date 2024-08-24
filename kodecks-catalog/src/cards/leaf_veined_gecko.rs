use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "leaf",
    "Leaf-Veined Gecko",
    color: Color::JADE,
    cost: 0,
    card_type: CardType::Creature,
    power: 100,
);

impl Effect for CardDef {}
