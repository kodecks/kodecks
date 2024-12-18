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
    power: 1,
    shards: 1,
);

impl Effect for CardDef {}
