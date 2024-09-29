#![forbid(unsafe_code)]

use bevy::{prelude::*, window::WindowTheme};

mod assets;
mod input;
mod painter;
mod scene;

fn main() {
    let mut app = App::new();

    #[cfg(feature = "embed_assets")]
    let app = app.add_plugins(bevy_embedded_assets::EmbeddedAssetPlugin {
        mode: bevy_embedded_assets::PluginMode::ReplaceAndFallback {
            path: "assets".to_string(),
        },
    });

    let default_plugins = DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Kodecks".into(),
                window_theme: Some(WindowTheme::Dark),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                resize_constraints: WindowResizeConstraints {
                    min_width: 1100.0,
                    min_height: 600.0,
                    ..default()
                },
                ..default()
            }),
            ..default()
        });

    app.add_plugins(default_plugins)
        .add_plugins((input::InputPlugin, scene::ScenePlugin))
        .run();
}
