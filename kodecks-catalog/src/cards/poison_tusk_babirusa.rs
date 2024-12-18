use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "pois",
    "Poison-Tusk Babirusa",
    color: Color::GREEN,
    cost: 4,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 200,
    abilities: &[KeywordAbility::Toxic][..],
    shards: 1,
);

impl Effect for CardDef {}
