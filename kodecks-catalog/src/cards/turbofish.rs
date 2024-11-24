use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "turb",
    "Turbofish",
    color: Color::BLUE,
    cost: 0,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 100,
    shards: 4,
);

impl Effect for CardDef {}
