use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "subs",
    "Subspace Mole",
    color: Color::BLUE,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Program,
    power: 300,
    shards: 1,
);

impl Effect for CardDef {}
