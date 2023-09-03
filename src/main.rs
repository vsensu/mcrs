use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::common_conditions::input_toggle_active;
use bevy::pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::prelude::*;
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
                }),
            WireframePlugin,
        ))
        .add_plugins(
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Grave)),
        )
        .add_plugins(LookTransformPlugin)
        .add_plugins(FpsCameraPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        // .add_plugins(EguiPlugin)
        .add_plugins(MaterialPlugin::<mcrs::ArrayTextureMaterial>::default())
        // .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, mcrs::setup)
        .add_systems(PostStartup, mcrs::post_setup)
        // .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, mcrs::input_mode)
        .init_resource::<mcrs::MouseSettings>()
        .register_type::<mcrs::MouseSettings>()
        .init_resource::<mcrs::DebugSettings>() // `ResourceInspectorPlugin` won't initialize the resource
        .register_type::<mcrs::DebugSettings>() // you need to register your type to display it
        // .add_plugins(ResourceInspectorPlugin::<mcrs::DebugSettings>::default()) // seperate window for the resource
        .add_systems(Update, mcrs::debug_system)
        .add_systems(Update, mcrs::fps)
        .add_systems(PreUpdate, mcrs::gen_chunks_data)
        .add_systems(Update, mcrs::update_column_meshes)
        .add_systems(Update, mcrs::load_chunks_around)
        .add_systems(Update, mcrs::handle_chunk_meshes_update_queue)
        .add_systems(Update, mcrs::create_array_texture)
        .add_systems(Update, mcrs::handle_voxel_modify_queue)
        .add_systems(Update, mcrs::hit_voxel)
        .run();
}
