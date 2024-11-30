use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "cyan",
    "Cyanide Spitter",
    color: Color::RED,
    cost: 3,
    card_type: CardType::Creature,
    creature_type: CreatureType::Cyborg,
    power: 200,
    abilities: &[KeywordAbility::Toxic][..],
);

impl Effect for CardDef {}
