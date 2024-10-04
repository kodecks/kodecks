use bevy::prelude::*;

pub fn get() -> StartupOptions {
    #[cfg(not(target_family = "wasm"))]
    {
        startup_options().run()
    }

    #[cfg(target_family = "wasm")]
    StartupOptions::default()
}

#[derive(Clone, Debug, Default, Resource)]
#[cfg_attr(
    not(target_family = "wasm"),
    derive(bpaf::Bpaf),
    bpaf(options, version)
)]
pub struct StartupOptions {
    #[cfg(not(target_family = "wasm"))]
    #[bpaf(argument("DIR"), env("KODECKS_DATA_DIR"))]
    /// The directory to store data in.
    pub data_dir: Option<std::path::PathBuf>,
}
