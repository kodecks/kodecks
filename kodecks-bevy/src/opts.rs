use bevy::prelude::*;

pub fn get() -> StartupOptions {
    #[cfg(not(target_arch = "wasm32"))]
    {
        startup_options().run()
    }

    #[cfg(target_arch = "wasm32")]
    StartupOptions::default()
}

#[derive(Clone, Debug, Default, Resource)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(bpaf::Bpaf),
    bpaf(options, version)
)]
pub struct StartupOptions {
    #[cfg(not(target_arch = "wasm32"))]
    #[bpaf(argument("DIR"), env("KODECKS_DATA_DIR"))]
    /// The directory to store data in.
    pub data_dir: Option<std::path::PathBuf>,
}
