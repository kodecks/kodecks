use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "demi",
    "Demilune Nighthawk",
    color: Color::BLUE,
    cost: 1,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 2,
    shards: 1,
);

impl Effect for CardDef {}
