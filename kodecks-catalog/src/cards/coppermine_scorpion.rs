use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "copp",
    "Coppermine Scorpion",
    color: Color::RED,
    cost: 2,
    card_type: CardType::Creature,
    power: 200,
    abilities: &[KeywordAbility::Toxic][..],
);

impl Effect for CardDef {}
