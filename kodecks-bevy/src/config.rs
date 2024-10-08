use crate::opts::StartupOptions;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use unic_langid::LanguageIdentifier;
use url::Url;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init).add_systems(
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

fn init(mut commands: Commands, opts: Res<StartupOptions>) {
    commands.insert_resource(GlobalConfig::load(&opts));
}

impl GlobalConfig {
    pub fn load(opts: &StartupOptions) -> Self {
        io::read_config(opts)
    }
}

fn write_config(config: Res<GlobalConfig>, opts: Res<StartupOptions>) {
    if *config == GlobalConfig::default() {
        return;
    }
    io::write_config(&config, &opts);
}

#[cfg(not(target_family = "wasm"))]
mod io {
    use super::GlobalConfig;
    use crate::opts::StartupOptions;
    use std::path::PathBuf;
    use tracing::{error, info};

    const FILENAME: &str = "config.json";

    fn get_config_path(opts: &StartupOptions) -> Option<PathBuf> {
        if let Some(data_dir) = &opts.data_dir {
            Some(data_dir.join(FILENAME))
        } else {
            let exe_path = std::env::current_exe().ok()?;
            Some(exe_path.parent()?.join(FILENAME))
        }
    }

    pub fn read_config(opts: &StartupOptions) -> GlobalConfig {
        let path = match get_config_path(opts) {
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

    pub fn write_config(config: &GlobalConfig, opts: &StartupOptions) {
        let path = match get_config_path(opts) {
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

#[cfg(target_family = "wasm")]
mod io {
    use super::GlobalConfig;
    use crate::opts::StartupOptions;
    use gloo_storage::{LocalStorage, Storage};
    use tracing::{error, warn};

    const KEY: &str = "config";

    pub fn read_config(_opts: &StartupOptions) -> GlobalConfig {
        match LocalStorage::get(KEY) {
            Ok(config) => config,
            Err(err) => {
                warn!("Could not load config: {}", err);
                GlobalConfig::default()
            }
        }
    }

    pub fn write_config(config: &GlobalConfig, _opts: &StartupOptions) {
        if let Err(err) = LocalStorage::set(KEY, config) {
            error!("Could not save config: {}", err);
        }
    }
}
