use core::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(transparent)]
pub struct Token(String);

impl Token {
    pub fn new() -> Self {
        Token(nanoid::nanoid!())
    }

    pub fn from_str(s: &str) -> Self {
        Token(s.to_string())
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Token {
    fn from(s: String) -> Self {
        Token(s)
    }
}
