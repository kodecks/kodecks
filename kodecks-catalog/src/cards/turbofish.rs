use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "turb",
    "Turbofish",
    color: Color::AZURE,
    cost: 0,
    card_type: CardType::Creature,
    power: 100,
);

impl Effect for CardDef {}
