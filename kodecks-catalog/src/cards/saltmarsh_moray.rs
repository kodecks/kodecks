use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "salt",
    "Saltmarsh Moray",
    color: Color::BLUE,
    cost: 3,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 4,
    abilities: &[KeywordAbility::Stealth][..],
    shards: 1,
);

impl Effect for CardDef {}
