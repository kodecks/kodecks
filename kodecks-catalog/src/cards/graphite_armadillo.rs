use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "grap",
    "Graphite Armadillo",
    color: Color::RED,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 2,
);

impl Effect for CardDef {}
