use bitflags::bitflags;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
    pub struct Color: u8 {
        const COLORLESS = 0b00000000;
        const RUBY = 0b00000001;
        const TOPAZ = 0b00000010;
        const JADE = 0b00000100;
        const AZURE = 0b00001000;
    }
}

impl Color {
    pub fn iter_all() -> impl Iterator<Item = Color> {
        [Color::RUBY, Color::TOPAZ, Color::JADE, Color::AZURE]
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
            if self.contains(Color::RUBY) {
                if !first {
                    write!(f, "+")?;
                }
                write!(f, "Ruby")?;
                first = false;
            }
            if self.contains(Color::TOPAZ) {
                if !first {
                    write!(f, "+")?;
                }
                write!(f, "Topaz")?;
                first = false;
            }
            if self.contains(Color::JADE) {
                if !first {
                    write!(f, "+")?;
                }
                write!(f, "Jade")?;
                first = false;
            }
            if self.contains(Color::AZURE) {
                if !first {
                    write!(f, "+")?;
                }
                write!(f, "Azure")?;
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
                "ruby" => color |= Color::RUBY,
                "topaz" => color |= Color::TOPAZ,
                "jade" => color |= Color::JADE,
                "azure" => color |= Color::AZURE,
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
