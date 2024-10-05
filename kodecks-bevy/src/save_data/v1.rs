use super::container::VersionTag;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use k256::schnorr::SigningKey;
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use std::{fmt, hash::Hash};

#[derive(Debug, Clone, DefaultFromSerde, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SaveDataV1 {
    version: VersionTag<1>,
    pub auth: Auth,
    pub statistics: Statistics,
}

#[derive(Debug, Clone, DefaultFromSerde, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Statistics {
    pub games: u32,
}

#[derive(Clone, DefaultFromSerde, Serialize, Deserialize)]
pub struct Auth {
    #[serde(
        default = "private_key_default",
        serialize_with = "serialize_private_key",
        deserialize_with = "deserialize_private_key"
    )]
    pub private_key: SigningKey,
}

impl PartialEq for Auth {
    fn eq(&self, other: &Self) -> bool {
        self.private_key.to_bytes() == other.private_key.to_bytes()
    }
}

impl Eq for Auth {}

impl Hash for Auth {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.private_key.verifying_key().to_bytes().hash(state);
    }
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Auth").finish()
    }
}

fn serialize_private_key<S>(private_key: &SigningKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let private_key = private_key.to_bytes();
    serializer.serialize_str(&URL_SAFE.encode(&*private_key))
}

fn deserialize_private_key<'de, D>(deserializer: D) -> Result<SigningKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let private_key = String::deserialize(deserializer)?;
    let private_key = URL_SAFE
        .decode(private_key.as_bytes())
        .map_err(serde::de::Error::custom)?;
    SigningKey::from_bytes(&private_key).map_err(serde::de::Error::custom)
}

fn private_key_default() -> SigningKey {
    let mut rng = rand::thread_rng();
    SigningKey::random(&mut rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let data = SaveDataV1::default();
        let json = serde_json::to_string(&data).unwrap();
        let decoded = serde_json::from_str::<SaveDataV1>(&json).unwrap();
        assert_eq!(data, decoded);
    }
}
