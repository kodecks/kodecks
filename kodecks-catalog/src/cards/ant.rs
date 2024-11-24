use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "ant",
    "Ant",
    color: Color::GREEN,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 100,
    is_token: true,
    shards: 3,
);

impl Effect for CardDef {}
