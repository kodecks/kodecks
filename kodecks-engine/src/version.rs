use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub struct VersionTag<const V: u8>;

impl<const V: u8> Serialize for VersionTag<V> {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(V)
    }
}

impl<'de, const V: u8> Deserialize<'de> for VersionTag<V> {
    fn deserialize<D: serde::de::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<VersionTag<V>, D::Error> {
        let version = u8::deserialize(deserializer)?;
        if version != V {
            return Err(serde::de::Error::custom(format!(
                "Expected version {}, got {}",
                V, version
            )));
        }
        Ok(VersionTag)
    }
}
