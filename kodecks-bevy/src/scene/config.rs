use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use unic_langid::{langid, LanguageIdentifier};

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub lang: LanguageIdentifier,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            lang: langid!("en-US"),
        }
    }
}
