use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "elfl",
    "Electric Flytrap",
    color: Color::GREEN,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 200,
    abilities: &[KeywordAbility::Devour][..],
);

impl Effect for CardDef {}
