use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "moss",
    "Moss-Grown Mastodon",
    color: Color::GREEN,
    cost: 7,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 700,
);

impl Effect for CardDef {}
