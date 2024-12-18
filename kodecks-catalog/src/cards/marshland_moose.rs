use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "mars",
    "Marshland Moose",
    color: Color::GREEN,
    cost: 4,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 400,
    shards: 1,
);

impl Effect for CardDef {}
