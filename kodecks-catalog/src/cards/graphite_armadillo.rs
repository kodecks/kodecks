use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "grap",
    "Graphite Armadillo",
    color: Color::RED,
    cost: 2,
    card_type: CardType::Creature,
    power: 200,
    shields: 1,
);

impl Effect for CardDef {}
