use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "wind",
    "Wind-Up Spider",
    color: Color::RED,
    cost: 0,
    card_type: CardType::Creature,
    creature_type: CreatureType::Robot,
    power: 100,
    shards: 2,
);

impl Effect for CardDef {}
