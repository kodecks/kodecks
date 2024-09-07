#![forbid(unsafe_code)]

use bevy::{prelude::*, window::WindowTheme};

mod assets;
mod painter;
mod scene;

fn main() {
    let mut app = App::new();

    let default_plugins = DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Kodecks".into(),
                window_theme: Some(WindowTheme::Dark),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        });

    app.add_plugins(default_plugins)
        .add_plugins(scene::ScenePlugin)
        .run();
}
