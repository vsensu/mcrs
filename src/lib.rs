mod voxel;

use std::f32::consts::PI;

use bevy::{
    asset::LoadState,
    diagnostic::{Diagnostics, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat},
    window::PrimaryWindow,
};
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};

use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use voxel::{Chunk, ChunkColumn, ChunkData, ChunkIndex, ChunkMesh};

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
pub struct Shape;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Start loading the texture.
    commands.insert_resource(LoadingTexture {
        is_loaded: false,
        handle: asset_server.load("textures/array_texture.png"),
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_x(-PI / 4.0)),
        ..default()
    });

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

    (0..voxel::INIT_WORLD_SIZE).for_each(|x| {
        (0..voxel::INIT_WORLD_SIZE).for_each(|z| {
            (0..voxel::CHUNK_LIMIT_Y).for_each(|y| {
                commands.spawn((
                    Chunk {
                        index: ChunkIndex {
                            x: x as i32,
                            y: y as i32,
                            z: z as i32,
                        },
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
    commands.insert_resource(voxel::VoxelData::default());
    commands.insert_resource(voxel::VoxelMeshes::default());
    commands.insert_resource(voxel::ChunkMeshesUpdateQueue::default());
    commands.insert_resource(VoxelMaterial::default());
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

pub fn gen_chunks_data(
    // mut commands: Commands,
    mut query: Query<&Chunk>,
    mut voxel_data: ResMut<voxel::VoxelData>,
    mut chunk_meshes_update_queue: ResMut<voxel::ChunkMeshesUpdateQueue>,
) {
    for chunk in query.iter_mut() {
        voxel_data.chunks.entry(chunk.index).or_insert_with(|| {
            chunk_meshes_update_queue.queue.insert(ChunkColumn {
                x: chunk.index.x,
                z: chunk.index.z,
            });
            println!(
                "Chunk {}_{}_{} generated",
                chunk.index.x, chunk.index.y, chunk.index.z
            );
            ChunkData::new(chunk.index)
        });
    }
}

pub fn update_column_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(Entity, &mut voxel::ColumnMesh)>,
    voxel_material: Res<VoxelMaterial>,
    voxel_data: Res<voxel::VoxelData>,
) {
    for (column_mesh_entity, mut column_mesh) in query.iter_mut() {
        if column_mesh.dirty {
            let mut chunk_num = 0;
            (0..voxel::CHUNK_LIMIT_Y).for_each(|i| {
                if voxel_data.chunks.contains_key(&ChunkIndex {
                    x: column_mesh.column.x,
                    y: i as i32,
                    z: column_mesh.column.z,
                }) {
                    chunk_num += 1;
                }
            });
            if chunk_num == voxel::CHUNK_LIMIT_Y {
                let mut chunks_mesh_data = Vec::new();
                (0..voxel::CHUNK_LIMIT_Y).for_each(|i| {
                    if let Some(chunk_data) = voxel_data.chunks.get(&ChunkIndex {
                        x: column_mesh.column.x,
                        y: i as i32,
                        z: column_mesh.column.z,
                    }) {
                        chunks_mesh_data.push(voxel::greedy_meshing(chunk_data));
                    }
                });
                meshes.remove(column_mesh.mesh.clone());
                column_mesh.mesh = meshes.add(voxel::combine_meshes(&chunks_mesh_data).into());
                commands
                    .entity(column_mesh_entity)
                    .insert(MaterialMeshBundle {
                        mesh: column_mesh.mesh.clone(),
                        material: voxel_material.material.clone(),
                        ..default()
                    });
                column_mesh.dirty = false;
                println!(
                    "ColumnMesh {}_{} updated",
                    column_mesh.column.x, column_mesh.column.z
                );
            }
        }
    }
}

pub fn load_chunks_around(
    mut commands: Commands,
    fps_camera_query: Query<&GlobalTransform, With<FpsCameraController>>,
    voxel_data: Res<voxel::VoxelData>,
) {
    let transform = fps_camera_query.single();
    let camera_pos = transform.translation();
    let chunk_index = voxel::get_chunk_index(&camera_pos);
    let sight_range = 4; // 4 chunks around the player
    for x in -sight_range..=sight_range {
        for z in -sight_range..=sight_range {
            (0..voxel::CHUNK_LIMIT_Y).for_each(|y| {
                let chunk_index_to_load = ChunkIndex {
                    x: chunk_index.x + x,
                    y: y as i32,
                    z: chunk_index.z + z,
                };
                if !voxel_data.chunks.contains_key(&chunk_index_to_load) {
                    commands.spawn((
                        Chunk {
                            index: chunk_index_to_load,
                        },
                        Name::new(format!(
                            "Chunk {}_{}_{}",
                            chunk_index_to_load.x, chunk_index_to_load.y, chunk_index_to_load.z
                        )),
                    ));
                }
            });
        }
    }
}

pub fn handle_chunk_meshes_update_queue(
    mut commands: Commands,
    mut chunk_meshes_update_queue: ResMut<voxel::ChunkMeshesUpdateQueue>,
    mut column_meshes: ResMut<voxel::VoxelMeshes>,
) {
    for chunk_column in chunk_meshes_update_queue.queue.iter() {
        if !column_meshes.columns.contains_key(chunk_column) {
            column_meshes.columns.insert(
                *chunk_column,
                commands
                    .spawn((Name::new(format!(
                        "ColumnMesh {}_{}",
                        chunk_column.x, chunk_column.z
                    )),))
                    .id(),
            );
        }
        let chunk_column_entity = column_meshes.columns.get(chunk_column).unwrap();
        commands
            .entity(*chunk_column_entity)
            .insert(voxel::ColumnMesh {
                column: *chunk_column,
                dirty: true,
                mesh: Default::default(),
            });
    }
    chunk_meshes_update_queue.queue.clear();
}

#[derive(Resource)]
pub struct LoadingTexture {
    is_loaded: bool,
    handle: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct VoxelMaterial {
    material: Handle<ArrayTextureMaterial>,
}

pub fn create_array_texture(
    asset_server: Res<AssetServer>,
    mut loading_texture: ResMut<LoadingTexture>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ArrayTextureMaterial>>,
    mut voxel_material: ResMut<VoxelMaterial>,
) {
    if loading_texture.is_loaded
        || asset_server.get_load_state(loading_texture.handle.clone()) != LoadState::Loaded
    {
        return;
    }
    loading_texture.is_loaded = true;
    let image = images.get_mut(&loading_texture.handle).unwrap();

    // Create a new array texture asset from the loaded texture.
    let array_layers = 4;
    image.reinterpret_stacked_2d_as_array(array_layers);

    let material_handle = materials.add(ArrayTextureMaterial {
        array_texture: loading_texture.handle.clone(),
    });
    voxel_material.material = material_handle;
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid, TypePath)]
#[uuid = "9c5a0ddf-1eaf-41b4-9832-ed736fd26af3"]
pub struct ArrayTextureMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    array_texture: Handle<Image>,
}

impl Material for ArrayTextureMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/array_texture.wgsl".into()
    }
}
