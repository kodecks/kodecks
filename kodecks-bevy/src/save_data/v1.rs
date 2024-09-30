use super::container::VersionTag;
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;

#[derive(Debug, Clone, DefaultFromSerde, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveDataV1 {
    version: VersionTag<1>,
    pub statistics: Statistics,
}

#[derive(Debug, Clone, DefaultFromSerde, PartialEq, Eq, Serialize, Deserialize)]
pub struct Statistics {
    pub games: u32,
}
