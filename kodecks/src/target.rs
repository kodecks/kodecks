use crate::{id::ObjectId, player::PlayerId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Player(PlayerId),
    Card(ObjectId),
}
