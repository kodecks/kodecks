use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "ruby",
    "Ruby Digger",
    color: Color::RED,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Robot,
    power: 300,
    shards: 3,
);

impl Effect for CardDef {}
