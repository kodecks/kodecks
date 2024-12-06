use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "moon",
    "Moonlit Gecko",
    color: Color::GREEN,
    cost: 1,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 1,
);

impl Effect for CardDef {}
