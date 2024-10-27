use crate::{player::PlayerConfig, pool::CardPool, regulation::Regulation};
use bincode::{
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{DecodeError, EncodeError},
    BorrowDecode, Decode, Encode,
};
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct GameProfile {
    pub regulation: Regulation,
    pub card_pool: CardPool,
    pub debug: Option<DebugConfig>,
    pub players: Vec<PlayerConfig>,
    pub bots: Vec<BotConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rng_seed: Option<u64>,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Encode, Decode)]
pub struct DebugConfig {
    pub no_deck_shuffle: bool,
    pub no_player_shuffle: bool,
    pub flags: DebugFlags,
}

bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DebugFlags: u8 {
        const DEBUG_COMMAND = 0b00000001;
        const IGNORE_COST = 0b00000010;
    }
}

impl Encode for DebugFlags {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.bits(), encoder)?;
        Ok(())
    }
}

impl Decode for DebugFlags {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self::from_bits_truncate(Decode::decode(decoder)?))
    }
}

impl<'de> BorrowDecode<'de> for DebugFlags {
    fn borrow_decode<D: BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self::from_bits_truncate(Decode::decode(decoder)?))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BotConfig {
    pub player: u8,
}
