use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "awkw",
    "Awkward Auk",
    color: Color::BLUE,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 3,
);

impl Effect for CardDef {}
