use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use flate2::Compression;
use flate2::{bufread::ZlibDecoder, write::ZlibEncoder};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;

const NONCE_LEN: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SaveData {
    V1(super::v1::SaveDataV1),
}

impl Default for SaveData {
    fn default() -> Self {
        SaveData::V1(Default::default())
    }
}

impl SaveData {
    pub fn encode(&self) -> anyhow::Result<String> {
        let mut nonce = vec![0; NONCE_LEN];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut nonce);

        let json = serde_json::to_string(self)?;
        let mut e = ZlibEncoder::new(nonce.clone(), Compression::default());
        e.write_all(json.as_bytes())?;

        let mut data = e.finish()?;
        data.iter_mut()
            .skip(nonce.len())
            .zip(nonce.iter().cycle())
            .for_each(|(a, b)| *a ^= *b);

        let data = URL_SAFE.encode(data);
        Ok(format!("Do NOT share this data with anyone.\nIt contains a secret key to access your account on the server.\n\n{}", data))
    }

    pub fn decode(s: &str) -> anyhow::Result<Self> {
        let data = s
            .trim()
            .rsplit(|c: char| c.is_ascii_whitespace())
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid data"))?;
        let data = URL_SAFE.decode(data.as_bytes())?;
        if data.len() < NONCE_LEN {
            return Err(anyhow::anyhow!("Invalid data length"));
        }
        let nonce = &data[..NONCE_LEN];
        let mut data = data[NONCE_LEN..].to_vec();
        data.iter_mut()
            .zip(nonce.iter().cycle())
            .for_each(|(a, b)| *a ^= *b);

        let d = ZlibDecoder::new(&data[..]);
        Ok(serde_json::from_reader(d)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_data() {
        let data = SaveData::default();
        let encoded = data.encode().unwrap();
        SaveData::decode(&encoded).unwrap();
    }
}
