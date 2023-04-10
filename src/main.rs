use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_startup_system(mcrs::setup)
        .add_system(bevy::window::close_on_esc)
        .add_system(mcrs::rotate)
        .add_system(mcrs::input_mode)
        .run();
}
