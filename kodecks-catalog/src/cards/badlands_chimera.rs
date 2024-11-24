use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "badl",
    "Badlands Chimera",
    color: Color::GREEN,
    cost: 5,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 700,
    shards: 6,
);

impl Effect for CardDef {}
