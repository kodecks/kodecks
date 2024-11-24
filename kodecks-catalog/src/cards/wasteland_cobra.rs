use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "wast",
    "Wasteland Cobra",
    color: Color::GREEN,
    cost: 1,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 200,
    shards: 1,
);

impl Effect for CardDef {}
