use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "elec",
    "Electric Clione",
    color: Color::BLUE,
    cost: 0,
    card_type: CardType::Creature,
    creature_type: CreatureType::Program,
    power: 100,
    abilities: &[KeywordAbility::Volatile][..],
);

impl Effect for CardDef {}
