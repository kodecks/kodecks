#![forbid(unsafe_code)]

use kodecks::card::Catalog;

mod cards;
mod macros;

pub static CATALOG: Catalog = Catalog {
    str_index: &cards::CARDS,
};
