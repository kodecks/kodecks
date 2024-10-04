use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use k256::schnorr::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub client_version: String,
    #[serde(flatten)]
    pub ty: LoginType,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum LoginType {
    PubkeyChallenge {
        #[serde(
            serialize_with = "serialize_pubkey",
            deserialize_with = "deserialize_pubkey"
        )]
        pubkey: VerifyingKey,
    },
    PubkeyResponse {
        #[serde(
            serialize_with = "serialize_pubkey",
            deserialize_with = "deserialize_pubkey"
        )]
        pubkey: VerifyingKey,
        #[serde(
            serialize_with = "serialize_signature",
            deserialize_with = "deserialize_signature"
        )]
        signature: Signature,
    },
}

fn serialize_pubkey<S>(pubkey: &VerifyingKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let bytes = pubkey.to_bytes();
    let s = URL_SAFE.encode(bytes);
    serializer.serialize_str(&s)
}

fn deserialize_pubkey<'de, D>(deserializer: D) -> Result<VerifyingKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let bytes = URL_SAFE.decode(s).map_err(serde::de::Error::custom)?;
    VerifyingKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
}

fn serialize_signature<S>(signature: &Signature, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let bytes = signature.to_bytes();
    let s = URL_SAFE.encode(bytes);
    serializer.serialize_str(&s)
}

fn deserialize_signature<'de, D>(deserializer: D) -> Result<Signature, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let bytes = URL_SAFE.decode(s).map_err(serde::de::Error::custom)?;
    Signature::try_from(bytes.as_slice()).map_err(serde::de::Error::custom)
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum LoginResponse {
    Session { token: String },
    Challenge { challenge: String },
    Failed,
}
