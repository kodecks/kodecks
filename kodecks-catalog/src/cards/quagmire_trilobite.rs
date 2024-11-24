use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "quag",
    "Quagmire Trilobite",
    color: Color::GREEN,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 300,
    shards: 6,
);

impl Effect for CardDef {}
