use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "quic",
    "Quicksand Skulker",
    color: Color::GREEN,
    cost: 4,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 5,
    abilities: &[KeywordAbility::Stealth][..],
    shards: 1,
);

impl Effect for CardDef {}
