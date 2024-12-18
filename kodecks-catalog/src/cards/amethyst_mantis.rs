use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "amet",
    "Amethyst Mantis",
    color: Color::RED,
    cost: 3,
    card_type: CardType::Creature,
    creature_type: CreatureType::Robot,
    power: 400,
    shards: 1,
);

impl Effect for CardDef {}
