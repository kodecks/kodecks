#![forbid(unsafe_code)]

use kodecks::card::Catalog;
use std::sync::{Arc, LazyLock};

mod cards;
pub mod decks;
mod macros;

pub static CATALOG: LazyLock<Arc<Catalog>> = LazyLock::new(|| Arc::new(Catalog::new(cards::CARDS)));
