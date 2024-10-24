use bincode::{
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{DecodeError, EncodeError},
    BorrowDecode, Decode, Encode,
};
use bitflags::bitflags;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Color: u8 {
        const COLORLESS = 0b00000000;
        const RED = 0b00000001;
        const YELLOW = 0b00000010;
        const GREEN = 0b00000100;
        const BLUE = 0b00001000;
    }
}

impl Color {
    pub fn iter_all() -> impl Iterator<Item = Color> {
        [Color::RED, Color::YELLOW, Color::GREEN, Color::BLUE]
            .iter()
            .copied()
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "Colorless")
        } else {
            let mut first = true;
            if self.contains(Color::RED) {
                if !first {
                    write!(f, "+")?;
                }
                write!(f, "Red")?;
                first = false;
            }
            if self.contains(Color::YELLOW) {
                if !first {
                    write!(f, "+")?;
                }
                write!(f, "Yellow")?;
                first = false;
            }
            if self.contains(Color::GREEN) {
                if !first {
                    write!(f, "+")?;
                }
                write!(f, "Green")?;
                first = false;
            }
            if self.contains(Color::BLUE) {
                if !first {
                    write!(f, "+")?;
                }
                write!(f, "Blue")?;
            }
            Ok(())
        }
    }
}

impl FromStr for Color {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Color, Self::Err> {
        let mut color = Color::empty();
        for part in s.to_ascii_lowercase().split('+') {
            match part.trim() {
                "red" => color |= Color::RED,
                "yellow" => color |= Color::YELLOW,
                "green" => color |= Color::GREEN,
                "blue" => color |= Color::BLUE,
                "colorless" => color = Color::empty(),
                _ => return Err("Invalid color"),
            }
        }
        Ok(color)
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().to_ascii_lowercase().as_str())
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Color::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Encode for Color {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.bits(), encoder)?;
        Ok(())
    }
}

impl Decode for Color {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self::from_bits_truncate(Decode::decode(decoder)?))
    }
}

impl<'de> BorrowDecode<'de> for Color {
    fn borrow_decode<D: BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self::from_bits_truncate(Decode::decode(decoder)?))
    }
}
