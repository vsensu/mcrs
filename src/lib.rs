mod voxel;

use std::f32::consts::PI;

use bevy::{
    diagnostic::{Diagnostics, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::PrimaryWindow,
};
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};

use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use voxel::ChunkIndex;

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
pub struct Shape;

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 128.0, 8.0),
        ..default()
    });

    (0..voxel::WORLD_SIZE).for_each(|x| {
        (0..voxel::WORLD_SIZE).for_each(|z| {
            (0..(voxel::HEIGHT_LIMIT / voxel::CHUNK_SIZE)).for_each(|y| {
                // commands.spawn(PbrBundle {
                //     mesh: meshes.add(
                //         voxel::ChunkData::new(ChunkIndex {
                //             x: x as i32,
                //             y: y as i32,
                //             z: z as i32,
                //         })
                //         .into(),
                //     ),
                //     material: materials.add(Color::SILVER.into()),
                //     ..default()
                // });
                commands.spawn((
                    PbrBundle {
                        mesh: meshes.add(voxel::greedy_meshing(&voxel::ChunkData::new(
                            ChunkIndex {
                                x: x as i32,
                                y: y as i32,
                                z: z as i32,
                            },
                        ))),
                        material: materials.add(Color::GREEN.into()),
                        // transform: Transform::from_xyz(16.0, 0.0, 0.0),
                        ..default()
                    },
                    Name::new(format!("Chunk {}_{}_{}", x, y, z)),
                ));
            });
        });
    });

    commands
        .spawn(Camera3dBundle::default())
        .insert(FpsCameraBundle::new(
            FpsCameraController::default(),
            Vec3::new(0.0, 128.0, 5.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::Y,
        ));

    commands.insert_resource(MouseSettings {
        speed: 10.0,
        sensitivity: Vec2::new(0.5, 0.5),
        ui_mode: true,
    });

    let text_section = move |color, value: &str| {
        TextSection::new(
            value,
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 20.0,
                color,
            },
        )
    };

    commands.spawn((
        TextBundle::from_sections([
            text_section(Color::GREEN, "FPS (raw): "),
            text_section(Color::CYAN, ""),
            text_section(Color::GREEN, "\nFPS (SMA): "),
            text_section(Color::CYAN, ""),
            text_section(Color::GREEN, "\nFPS (EMA): "),
            text_section(Color::CYAN, ""),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        StatsText,
    ));

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_sections([text_section(
                Color::WHITE,
                "Press ~ to toggle control mode",
            )]));
        });
}

pub fn post_setup(ms: Res<MouseSettings>, mut fps_camera_query: Query<&mut FpsCameraController>) {
    fps_camera_query.single_mut().enabled = !ms.ui_mode;
}

#[derive(Reflect, Resource, Default, Debug, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct MouseSettings {
    speed: f32,
    sensitivity: Vec2,
    ui_mode: bool,
}

pub fn input_mode(
    mut ms: ResMut<MouseSettings>,
    keyboard_input: Res<Input<KeyCode>>,
    mut primary_query: Query<&mut Window, With<PrimaryWindow>>,
    mut fps_camera_query: Query<&mut FpsCameraController>,
) {
    if keyboard_input.just_released(KeyCode::Grave) {
        ms.ui_mode = !ms.ui_mode;

        let mut fps_camera = fps_camera_query.single_mut();
        fps_camera.enabled = !ms.ui_mode;
        fps_camera.translate_sensitivity = ms.speed;
        fps_camera.mouse_rotate_sensitivity = ms.sensitivity;

        if let Ok(mut primary) = primary_query.get_single_mut() {
            primary.cursor.visible = ms.ui_mode;
        };
    }

    let Ok(mut primary) = primary_query.get_single_mut() else {
        return;
    };

    if !ms.ui_mode {
        let size = Vec2 {
            x: primary.width(),
            y: primary.height(),
        };
        let center = size / 2.0;
        primary.set_cursor_position(Some(center));
    }
}

// `InspectorOptions` are completely optional
#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct DebugSettings {
    wireframe: bool,
}

pub fn debug_system(
    debug_settings: Res<DebugSettings>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    wireframe_config.global = debug_settings.wireframe;
}

#[derive(Component)]
pub struct StatsText;

pub fn fps(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<StatsText>>) {
    let mut text = query.single_mut();

    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(raw) = fps.value() {
            text.sections[1].value = format!("{raw:.2}");
        }
        if let Some(sma) = fps.average() {
            text.sections[3].value = format!("{sma:.2}");
        }
        if let Some(ema) = fps.smoothed() {
            text.sections[5].value = format!("{ema:.2}");
        }
    };
}
