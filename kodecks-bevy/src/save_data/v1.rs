use super::container::VersionTag;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;

#[derive(Debug, Clone, DefaultFromSerde, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveDataV1 {
    version: VersionTag<1>,
    pub auth: Auth,
    pub statistics: Statistics,
}

#[derive(Debug, Clone, DefaultFromSerde, PartialEq, Eq, Serialize, Deserialize)]
pub struct Statistics {
    pub games: u32,
}

#[derive(Debug, Clone, DefaultFromSerde, PartialEq, Eq, Serialize, Deserialize)]
pub struct Auth {
    #[serde(
        default = "private_key_default",
        serialize_with = "serialize_private_key",
        deserialize_with = "deserialize_private_key"
    )]
    pub private_key: k256::SecretKey,
}

fn serialize_private_key<S>(private_key: &k256::SecretKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let private_key = private_key.to_bytes();
    serializer.serialize_str(&URL_SAFE.encode(&*private_key))
}

fn deserialize_private_key<'de, D>(deserializer: D) -> Result<k256::SecretKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let private_key = String::deserialize(deserializer)?;
    let private_key = URL_SAFE
        .decode(private_key.as_bytes())
        .map_err(serde::de::Error::custom)?;
    k256::SecretKey::from_slice(&private_key).map_err(serde::de::Error::custom)
}

fn private_key_default() -> k256::SecretKey {
    let mut rng = rand::thread_rng();
    k256::SecretKey::random(&mut rng)
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
