use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::render::{render_resource::WgpuFeatures, settings::WgpuSettings, RenderPlugin};
use bevy::window::PresentMode;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    wgpu_settings: WgpuSettings {
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    },
                }),
            WireframePlugin,
        ))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(LookTransformPlugin)
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        // .add_plugins(EguiPlugin)
        .add_systems(Startup, mcrs::setup)
        .add_systems(PostStartup, mcrs::post_setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, mcrs::input_mode)
        .init_resource::<mcrs::DebugSettings>() // `ResourceInspectorPlugin` won't initialize the resource
        .register_type::<mcrs::DebugSettings>() // you need to register your type to display it
        .add_plugins(ResourceInspectorPlugin::<mcrs::DebugSettings>::default())
        .add_systems(Update, mcrs::debug_system)
        .add_systems(Update, mcrs::fps)
        .run();
}
