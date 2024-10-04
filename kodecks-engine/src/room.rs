use bincode::{Decode, Encode};
use kodecks::regulation::Regulation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, Hash)]
pub struct RoomConfig {
    pub regulation: Regulation,
    pub room_type: RoomType,
}

#[derive(
    Debug, Clone, Default, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, Hash,
)]
pub enum RoomType {
    #[default]
    RandomMatch,
}
