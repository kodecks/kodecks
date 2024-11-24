use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "ceno",
    "Cenote Otter",
    color: Color::GREEN,
    cost: 3,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 400,
    shards: 2,
);

impl Effect for CardDef {}
