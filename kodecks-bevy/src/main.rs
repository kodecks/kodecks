#![forbid(unsafe_code)]

use bevy::{asset::AssetMetaCheck, prelude::*, window::WindowTheme};

mod assets;
mod config;
mod debugger;
mod engine;
mod input;
mod opts;
mod painter;
mod save_data;
mod scene;

fn main() {
    let opts = opts::get();

    let mut app = App::new();
    app.insert_resource(Msaa::Off);
    app.insert_resource(opts);

    #[cfg(feature = "embed_assets")]
    let app = app.add_plugins(bevy_embedded_assets::EmbeddedAssetPlugin {
        mode: bevy_embedded_assets::PluginMode::ReplaceAndFallback {
            path: "assets".to_string(),
        },
    });

    let default_plugins = DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Kodecks".into(),
                window_theme: Some(WindowTheme::Dark),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                resize_constraints: WindowResizeConstraints {
                    min_width: 1000.0,
                    min_height: 600.0,
                    ..default()
                },
                ..default()
            }),
            ..default()
        });

    let app = app
        .add_plugins(default_plugins)
        .add_plugins(debugger::DebuggerPlugin);

    app.add_plugins((
        input::InputPlugin,
        scene::ScenePlugin,
        save_data::SaveDataPlugin,
    ))
    .run();
}

pub fn app_version() -> String {
    if let Some(sha) = option_env!("VERGEN_GIT_SHA") {
        format!(
            "v{} ({}) {}",
            env!("CARGO_PKG_VERSION"),
            sha,
            env!("VERGEN_CARGO_TARGET_TRIPLE")
        )
    } else {
        format!(
            "v{} {}",
            env!("CARGO_PKG_VERSION"),
            env!("VERGEN_CARGO_TARGET_TRIPLE")
        )
    }
}
