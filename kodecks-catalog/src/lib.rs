#![forbid(unsafe_code)]

use kodecks::card::Catalog;
use std::sync::LazyLock;

mod cards;
pub mod decks;
mod macros;

pub static CATALOG: LazyLock<Catalog> = LazyLock::new(|| Catalog::new(cards::CARDS));
