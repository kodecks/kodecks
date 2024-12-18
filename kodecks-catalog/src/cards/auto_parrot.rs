use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "auto",
    "Auto Parrot",
    color: Color::BLUE,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 3,
    shards: 1,
);

impl Effect for CardDef {}
