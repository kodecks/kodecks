use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use unic_langid::LanguageIdentifier;
use url::Url;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalConfig::load()).add_systems(
            Update,
            write_config.run_if(resource_changed::<GlobalConfig>),
        );
    }
}

#[derive(Debug, Clone, DefaultFromSerde, Resource, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<LanguageIdentifier>,
    #[serde(default = "default_url", skip_serializing_if = "is_default_url")]
    pub server: Url,
}

fn default_url() -> Url {
    Url::parse("https://kodecks.onrender.com").unwrap()
}

fn is_default_url(url: &Url) -> bool {
    *url == default_url()
}

impl GlobalConfig {
    pub fn load() -> Self {
        io::read_config()
    }
}

fn write_config(config: Res<GlobalConfig>) {
    if *config == GlobalConfig::default() {
        return;
    }
    io::write_config(&config);
}

#[cfg(not(target_arch = "wasm32"))]
mod io {
    use super::GlobalConfig;
    use std::path::PathBuf;
    use tracing::{error, info};

    fn get_config_path() -> Option<PathBuf> {
        let exe_path = std::env::current_exe().ok()?;
        Some(exe_path.parent()?.join("config.json"))
    }

    pub fn read_config() -> GlobalConfig {
        let path = match get_config_path() {
            Some(path) => path,
            None => {
                error!("Could not find config file");
                return GlobalConfig::default();
            }
        };
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(err) => {
                info!("Could not open config file: {}", err);
                return GlobalConfig::default();
            }
        };
        match serde_json::from_reader(file) {
            Ok(config) => config,
            Err(err) => {
                error!("Could not parse config file: {}", err);
                GlobalConfig::default()
            }
        }
    }

    pub fn write_config(config: &GlobalConfig) {
        let path = match get_config_path() {
            Some(path) => path,
            None => {
                error!("Could not find config file");
                return;
            }
        };
        let file = match std::fs::File::create(path) {
            Ok(file) => file,
            Err(err) => {
                error!("Could not create config file: {}", err);
                return;
            }
        };
        if let Err(err) = serde_json::to_writer_pretty(file, config) {
            error!("Could not write config file: {}", err);
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod io {
    use super::GlobalConfig;
    use gloo_storage::{LocalStorage, Storage};
    use tracing::{error, warn};

    pub fn read_config() -> GlobalConfig {
        match LocalStorage::get("config") {
            Ok(config) => config,
            Err(err) => {
                warn!("Could not load config: {}", err);
                GlobalConfig::default()
            }
        }
    }

    pub fn write_config(config: &GlobalConfig) {
        if let Err(err) = LocalStorage::set("config", config) {
            error!("Could not save config: {}", err);
        }
    }
}
