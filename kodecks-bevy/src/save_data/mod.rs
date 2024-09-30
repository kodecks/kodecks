use bevy::prelude::*;
use futures::{
    channel::mpsc::{self, Sender},
    StreamExt,
};

mod container;
mod v1;

pub struct SaveDataPlugin;

impl Plugin for SaveDataPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DataWriter>()
            .insert_resource(SaveData::load())
            .add_systems(Update, write_data.run_if(resource_changed::<SaveData>));
    }
}

#[derive(Debug, Clone, Default, Resource, Deref, DerefMut, PartialEq, Eq)]
pub struct SaveData(v1::SaveDataV1);

impl SaveData {
    pub fn load() -> Self {
        io::read_data()
    }
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
    #[cfg(not(target_arch = "wasm32"))]
    task: bevy::tasks::Task<()>,
}

impl Default for DataWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl DataWriter {
    fn new() -> Self {
        let (send, mut recv) = mpsc::channel(256);
        let task = bevy::tasks::IoTaskPool::get().spawn(async move {
            while let Some(new_data) = recv.next().await {
                io::write_data(&new_data);
            }
        });
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self { send, task }
        }
        #[cfg(target_arch = "wasm32")]
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
        #[cfg(not(target_arch = "wasm32"))]
        bevy::tasks::block_on(&mut self.task);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod io {
    use super::{container, SaveData};
    use std::{fs, path::PathBuf};
    use tracing::{error, info};

    fn get_data_path() -> Option<PathBuf> {
        let exe_path = std::env::current_exe().ok()?;
        Some(exe_path.parent()?.join("save data.txt"))
    }

    pub fn read_data() -> SaveData {
        let path = match get_data_path() {
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

    pub fn write_data(data: &SaveData) {
        let path = match get_data_path() {
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

#[cfg(target_arch = "wasm32")]
mod io {
    use super::container;
    use super::SaveData;
    use gloo_storage::{LocalStorage, Storage};
    use tracing::{error, warn};

    pub fn read_data() -> SaveData {
        let data = match LocalStorage::get::<String>("save data") {
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

    pub fn write_data(data: &SaveData) {
        let data = match container::SaveData::V1(data.0.clone()).encode() {
            Ok(data) => data,
            Err(err) => {
                error!("Could not encode save data: {}", err);
                return;
            }
        };
        if let Err(err) = LocalStorage::set("save data", data) {
            error!("Could not save save data: {}", err);
        }
    }
}
