use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "soun",
    "Soundless Owl",
    color: Color::BLUE,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 3,
    abilities: &[KeywordAbility::Stealth][..],
    shards: 1,
);

impl Effect for CardDef {}
