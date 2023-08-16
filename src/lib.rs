mod voxel;

use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::PrimaryWindow,
};
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};
use voxel::ChunkIndex;

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
pub struct Shape;

pub fn setup(
    mut commands: Commands,
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
                commands.spawn(PbrBundle {
                    mesh: meshes.add(
                        voxel::ChunkData::new(ChunkIndex {
                            x: x as i32,
                            y: y as i32,
                            z: z as i32,
                        })
                        .into(),
                    ),
                    material: materials.add(Color::SILVER.into()),
                    ..default()
                });
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
        sensitivity: 0.02,
        ui_mode: true,
    });
}

pub fn post_setup(ms: Res<MouseSettings>, mut fps_camera_query: Query<&mut FpsCameraController>) {
    fps_camera_query.single_mut().enabled = !ms.ui_mode;
}

#[derive(Resource, Default, Debug)]
pub struct MouseSettings {
    speed: f32,
    sensitivity: f32,
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

        fps_camera_query.single_mut().enabled = !ms.ui_mode;

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
