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
    power: 1,
    shards: 1,
);

impl Effect for CardDef {}
