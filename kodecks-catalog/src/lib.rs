#![forbid(unsafe_code)]

use kodecks::card::Catalog;

mod cards;
pub mod decks;
mod macros;

pub static CATALOG: Catalog = Catalog {
    str_index: &cards::CARDS,
};
