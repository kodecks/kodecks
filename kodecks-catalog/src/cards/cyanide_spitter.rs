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
    power: 2,
    abilities: &[KeywordAbility::Toxic][..],
    shards: 1,
);

impl Effect for CardDef {}
