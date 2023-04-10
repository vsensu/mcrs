use bevy::prelude::*;
use mcrs;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(mcrs::setup)
        .add_system(bevy::window::close_on_esc)
        .add_system(mcrs::rotate)
        .add_system(mcrs::camera_movement)
        .add_system(mcrs::input_mode)
        .add_system(mcrs::mouse_look)
        .run();
}
