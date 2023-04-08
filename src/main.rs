use bevy::prelude::*;
use mcrs;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_startup_system(mcrs::setup)
        .add_system(mcrs::rotate)
        .add_system(mcrs::camera_movement)
        .add_system(mcrs::mouse_look)
        .run();
}
