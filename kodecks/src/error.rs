use crate::{color::Color, id::ObjectId};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Insufficient shards: {color} {amount}")]
    InsufficientShards { color: Color, amount: u32 },
    #[error("Creature already casted")]
    CreatureAlreadyCasted,
    #[error("Card not found: {id}")]
    CardNotFound { id: ObjectId },
    #[error("Key not found: {key}")]
    KeyNotFound { key: String },
    #[error("Invalid value type")]
    InvalidValueType,
}
