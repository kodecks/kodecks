use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "heli",
    "Helium Puffer",
    color: Color::BLUE,
    cost: 1,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 200,
);

impl Effect for CardDef {}
