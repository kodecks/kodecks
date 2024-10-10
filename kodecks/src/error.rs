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
    #[error("Client version outdated: server: {server} client: {client} req:{requirement}")]
    ClientVersionOutdated {
        server: String,
        client: String,
        requirement: String,
    },
    #[error("Server version outdated: server: {server} client: {client} req:{requirement}")]
    ServerVersionOutdated {
        server: String,
        client: String,
        requirement: String,
    },
}

impl<'a> From<Error> for Request<'a, FluentArgs<'a>> {
    fn from(error: Error) -> Request<'a, FluentArgs<'a>> {
        let mut args = FluentArgs::new();
        let id = match error {
            Error::FailedToConnectServer => "error-failed-to-connect-server",
            Error::ClientVersionOutdated { .. } => "error-client-version-outdated",
            Error::ServerVersionOutdated { .. } => "error-server-version-outdated",
        };
        match error {
            Error::FailedToConnectServer => {}
            Error::ClientVersionOutdated {
                server,
                client,
                requirement,
            } => {
                args.set("server", server.to_string());
                args.set("client", client.to_string());
                args.set("requirement", requirement.to_string());
            }
            Error::ServerVersionOutdated {
                server,
                client,
                requirement,
            } => {
                args.set("server", server.to_string());
                args.set("client", client.to_string());
                args.set("requirement", requirement.to_string());
            }
        }
        Request {
            id,
            attr: None,
            args: Some(args),
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
