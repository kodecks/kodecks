use crate::card_def;
use kodecks::prelude::*;

card_def!(
    CardDef,
    "icef",
    "Icefall Weasel",
    color: Color::BLUE,
    cost: 2,
    card_type: CardType::Creature,
    creature_type: CreatureType::Mutant,
    power: 2,
);

impl Effect for CardDef {}
