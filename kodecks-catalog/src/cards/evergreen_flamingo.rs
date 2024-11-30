use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "ever",
    "Evergreen Flamingo",
    color: Color::GREEN,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 200,
);

impl Effect for CardDef {}
