use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "vora",
    "Voracious Anteater",
    color: Color::GREEN,
    cost: 3,
    card_type: CardType::Creature,
    power: 400,
);

impl Effect for CardDef {}
