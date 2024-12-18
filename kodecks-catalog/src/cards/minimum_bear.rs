use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "mini",
    "Minimum Bear",
    color: Color::BLUE,
    cost: 0,
    card_type: CardType::Creature,
    creature_type: CreatureType::Program,
    power: 100,
    shards: 1,
);

impl Effect for CardDef {}
