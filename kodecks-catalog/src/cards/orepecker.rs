use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "orep",
    "Orepecker",
    color: Color::RED,
    cost: 1,
    card_type: CardType::Creature,
    creature_type: CreatureType::Robot,
    power: 100,
    abilities: &[KeywordAbility::Piercing][..],
);

impl Effect for CardDef {}
