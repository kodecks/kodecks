use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "ant",
    "Ant",
    color: Color::GREEN,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 1,
    is_token: true,
);

impl Effect for CardDef {}
