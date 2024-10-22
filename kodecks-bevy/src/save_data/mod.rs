use crate::opts::StartupOptions;
use bevy::prelude::*;
use futures::{
    channel::mpsc::{self, Sender},
    StreamExt,
};
use kodecks_catalog::{decks::starter_deck, CATALOG};

mod container;
mod v1;

pub struct SaveDataPlugin;

impl Plugin for SaveDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init)
            .add_systems(Update, write_data.run_if(resource_changed::<SaveData>));
    }
}

#[derive(Debug, Clone, Default, Resource, Deref, DerefMut, PartialEq, Eq, Hash)]
pub struct SaveData(v1::SaveDataV1);

impl SaveData {
    pub fn load(opts: &StartupOptions) -> Self {
        let mut data = io::read_data(opts);
        data.decks.list = vec![starter_deck()];
        data.inventory.cards = CATALOG.iter().map(|card| (card.id, 4)).collect();
        data
    }
}

fn init(mut commands: Commands, opts: Res<StartupOptions>) {
    commands.insert_resource(DataWriter::new(opts.clone()));
    commands.insert_resource(SaveData::load(&opts));
}

fn write_data(config: Res<SaveData>, mut data_writer: ResMut<DataWriter>) {
    if *config == SaveData::default() {
        return;
    }
    data_writer.write(config.clone());
}

#[derive(Resource)]
struct DataWriter {
    send: Sender<SaveData>,
    #[cfg(not(target_family = "wasm"))]
    task: bevy::tasks::Task<()>,
}

impl DataWriter {
    fn new(opts: StartupOptions) -> Self {
        let (send, mut recv) = mpsc::channel(256);
        let task = bevy::tasks::IoTaskPool::get().spawn(async move {
            while let Some(new_data) = recv.next().await {
                io::write_data(&new_data, &opts);
            }
        });
        #[cfg(not(target_family = "wasm"))]
        {
            Self { send, task }
        }
        #[cfg(target_family = "wasm")]
        {
            task.detach();
            Self { send }
        }
    }

    fn write(&mut self, data: SaveData) {
        let _ = self.send.try_send(data);
    }
}

impl Drop for DataWriter {
    fn drop(&mut self) {
        self.send.close_channel();
        #[cfg(not(target_family = "wasm"))]
        bevy::tasks::block_on(&mut self.task);
    }
}

#[cfg(not(target_family = "wasm"))]
mod io {
    use super::{container, SaveData};
    use crate::opts::StartupOptions;
    use std::{fs, path::PathBuf};
    use tracing::{error, info};

    const FILENAME: &str = "savedata.txt";

    fn get_data_path(opts: &StartupOptions) -> Option<PathBuf> {
        if let Some(dir) = &opts.data_dir {
            Some(dir.join(FILENAME))
        } else {
            let exe_path = std::env::current_exe().ok()?;
            Some(exe_path.parent()?.join(FILENAME))
        }
    }

    pub fn read_data(opts: &StartupOptions) -> SaveData {
        let path = match get_data_path(opts) {
            Some(path) => path,
            None => {
                error!("Could not find save data file");
                return SaveData::default();
            }
        };
        let data = match fs::read_to_string(path) {
            Ok(data) => data,
            Err(err) => {
                info!("Could not open save data file: {}", err);
                return SaveData::default();
            }
        };
        match container::SaveData::decode(&data) {
            Ok(container::SaveData::V1(config)) => SaveData(config),
            Err(err) => {
                error!("Could not parse save data file: {}", err);
                SaveData::default()
            }
        }
    }

    pub fn write_data(data: &SaveData, opts: &StartupOptions) {
        let path = match get_data_path(opts) {
            Some(path) => path,
            None => {
                error!("Could not find save data file");
                return;
            }
        };
        let data = match container::SaveData::V1(data.0.clone()).encode() {
            Ok(data) => data,
            Err(err) => {
                error!("Could not encode save data: {}", err);
                return;
            }
        };
        if let Err(err) = fs::write(path, data) {
            error!("Could not write save data file: {}", err);
        }
    }
}

#[cfg(target_family = "wasm")]
mod io {
    use super::container;
    use super::SaveData;
    use crate::opts::StartupOptions;
    use gloo_storage::{LocalStorage, Storage};
    use tracing::{error, warn};

    const KEY: &str = "savedata";

    pub fn read_data(_opts: &StartupOptions) -> SaveData {
        let data = match LocalStorage::get::<String>(KEY) {
            Ok(data) => data,
            Err(err) => {
                warn!("Could not load save data: {}", err);
                return SaveData::default();
            }
        };
        match container::SaveData::decode(&data) {
            Ok(container::SaveData::V1(config)) => SaveData(config),
            Err(err) => {
                error!("Could not parse save data file: {}", err);
                SaveData::default()
            }
        }
    }

    pub fn write_data(data: &SaveData, _opts: &StartupOptions) {
        let data = match container::SaveData::V1(data.0.clone()).encode() {
            Ok(data) => data,
            Err(err) => {
                error!("Could not encode save data: {}", err);
                return;
            }
        };
        if let Err(err) = LocalStorage::set(KEY, data) {
            error!("Could not save save data: {}", err);
        }
    }
}
