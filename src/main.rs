use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(LookTransformPlugin)
        .add_plugins(FpsCameraPlugin::default())
        .add_systems(Startup, mcrs::setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, mcrs::rotate)
        .add_systems(Update, mcrs::input_mode)
        .run();
}
