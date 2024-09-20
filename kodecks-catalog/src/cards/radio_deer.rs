use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "radi",
    "Radio Deer",
    color: Color::GREEN,
    cost: 3,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 200,
    shields: 1,
);

impl Effect for CardDef {}
