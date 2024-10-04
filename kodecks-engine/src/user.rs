use bech32::{Bech32m, Hrp};
use bincode::{Decode, Encode};
use core::fmt;
use k256::schnorr::VerifyingKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode, Hash)]
#[serde(transparent)]
pub struct UserId(String);

impl UserId {
    pub fn from_pubkey(pubkey: &VerifyingKey) -> Self {
        let hrp = Hrp::parse("ko").unwrap();
        let id = bech32::encode::<Bech32m>(hrp, &pubkey.to_bytes()).unwrap();
        UserId(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        UserId(s)
    }
}
