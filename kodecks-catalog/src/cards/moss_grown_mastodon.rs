use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "moss",
    "Moss-Grown Mastodon",
    color: Color::JADE,
    cost: 7,
    card_type: CardType::Creature,
    power: 700,
);

impl Effect for CardDef {}
