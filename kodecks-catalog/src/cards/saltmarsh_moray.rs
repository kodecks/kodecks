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
    power: 400,
    abilities: &[KeywordAbility::Stealth][..],
);

impl Effect for CardDef {}
