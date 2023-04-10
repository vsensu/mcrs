use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::PrimaryWindow,
};

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
pub struct Shape;

const X_EXTENT: f32 = 14.5;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let shapes = [
        meshes.add(shape::Cube::default().into()),
        meshes.add(shape::Box::default().into()),
        meshes.add(shape::Capsule::default().into()),
        meshes.add(shape::Torus::default().into()),
        meshes.add(shape::Cylinder::default().into()),
        meshes.add(shape::Icosphere::default().try_into().unwrap()),
        meshes.add(shape::UVSphere::default().into()),
    ];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            PbrBundle {
                mesh: shape,
                material: debug_material.clone(),
                transform: Transform::from_xyz(
                    -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                    2.0,
                    0.0,
                )
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                ..default()
            },
            Shape,
        ));
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: materials.add(Color::SILVER.into()),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });

    commands.insert_resource(MouseSettings {
        speed: 10.0,
        sensitivity: 0.02,
        ui_mode: true,
    });
}

pub fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}

#[derive(Resource, Default, Debug)]
pub struct MouseSettings {
    speed: f32,
    sensitivity: f32,
    ui_mode: bool,
}

pub fn camera_movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
    ms: Res<MouseSettings>,
) {
    let mut direction = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::W) {
        direction.z -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::A) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::S) {
        direction.z += 1.0;
    }
    if keyboard_input.pressed(KeyCode::D) {
        direction.x += 1.0;
    }
    if direction.length() > 0.0 {
        direction = direction.normalize();
    }
    let delta_seconds = time.delta_seconds();
    let translation = direction * ms.speed * delta_seconds;
    let mut transform = query.single_mut();
    let forward = transform.local_z();
    let right = transform.local_x();
    transform.translation += translation.x * right + translation.z * forward;
}

pub fn input_mode(
    mut ms: ResMut<MouseSettings>,
    keyboard_input: Res<Input<KeyCode>>,
    mut primary_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keyboard_input.just_released(KeyCode::Grave) {
        ms.ui_mode = !ms.ui_mode;

        let Ok(mut primary) = primary_query.get_single_mut() else {
            return;
        };

        primary.cursor.visible = ms.ui_mode;

        if !ms.ui_mode {
            let size = Vec2 {
                x: primary.width(),
                y: primary.height(),
            };
            let center = size / 2.0;
            primary.set_cursor_position(Some(center));
        }
    }
}

// first person camera
pub fn mouse_look(
    time: Res<Time>,
    ms: Res<MouseSettings>,
    mut primary_query: Query<&mut Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    if ms.ui_mode {
        return;
    }

    let Ok(mut primary) = primary_query.get_single_mut() else {
        return;
    };

    if !primary.focused {
        return;
    }

    let delta = time.delta_seconds();
    let size = Vec2 {
        x: primary.width(),
        y: primary.height(),
    };
    let center = size / 2.0;

    let mut delta_mouse = primary.cursor_position().unwrap_or(center) - center;
    if delta_mouse.length_squared() > 0.0 {
        let window_scale = primary.height().min(primary.width());
        delta_mouse *= ms.sensitivity * window_scale * delta;
        let mut transform = query.single_mut();
        let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        pitch -= -delta_mouse.y.to_radians();
        yaw -= delta_mouse.x.to_radians();
        pitch = pitch.clamp(-1.54, 1.54);
        transform.rotation =
            Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
    }
    primary.set_cursor_position(Some(center));
}
