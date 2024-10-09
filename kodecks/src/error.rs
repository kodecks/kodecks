use crate::{
    color::Color,
    id::{ObjectId, TimedObjectId},
};
use bincode::{Decode, Encode};
use fluent_bundle::FluentArgs;
use fluent_content::Request;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, Hash)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum Error {
    #[error("Failed to connect the server")]
    FailedToConnectServer,
}

impl<'a> From<Error> for Request<'a, FluentArgs<'a>> {
    fn from(_error: Error) -> Request<'a, FluentArgs<'a>> {
        Request {
            id: "error-failed-to-connect-server",
            attr: None,
            args: None,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ActionError {
    #[error("Insufficient shards: {color} {amount}")]
    InsufficientShards { color: Color, amount: u32 },
    #[error("Creature already free-casted")]
    CreatureAlreadyFreeCasted,
    #[error("Card not found: {id}")]
    CardNotFound { id: ObjectId },
    #[error("Key not found: {key}")]
    KeyNotFound { key: String },
    #[error("Invalid value type")]
    InvalidValueType,
    #[error("Target lost: {target}")]
    TargetLost { target: TimedObjectId },
}
