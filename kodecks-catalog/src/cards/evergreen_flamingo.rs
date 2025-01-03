use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "ever",
    "Evergreen Flamingo",
    color: Color::GREEN,
    cost: 1,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 200,
    shards: 3,
);

impl Effect for CardDef {}
