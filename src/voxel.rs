use std::collections::{HashMap, HashSet};

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};
use lerp::Lerp;
use noise::{NoiseFn, Perlin, Seedable};

pub const WORLD_SIZE: usize = 100; // 4 chunks in each direction
pub const INIT_WORLD_SIZE: usize = 4; // 4 chunks in each direction at the beginning
pub const CHUNK_SIZE: usize = 16; // 16 voxels in each direction
const WAVE_LENGTH: usize = 64; // voxel wave length in each direction
pub const CHUNK_LIMIT_Y: usize = 16; // chunk limit in y direction
pub const HEIGHT_LIMIT: usize = CHUNK_SIZE * CHUNK_LIMIT_Y; // height limit of the world

// cube cornors
const CORNORS: [Vec3; 8] = [
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(0.0, 1.0, 1.0),
    Vec3::new(0.0, 0.0, 1.0),
    Vec3::new(1.0, 0.0, 1.0),
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(1.0, 1.0, 0.0),
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(0.0, 0.0, 0.0),
];

// +x:0 +y:1 +z:2 -x:3 -y:4 -z:5 same as FaceDirection
const NORMALS: [Vec3; 6] = [
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(0.0, 0.0, 1.0),
    Vec3::new(-1.0, 0.0, 0.0),
    Vec3::new(0.0, -1.0, 0.0),
    Vec3::new(0.0, 0.0, -1.0),
];

const UVS: [Vec2; 4] = [
    Vec2::new(1.0, 0.0),
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, 1.0),
    Vec2::new(1.0, 1.0),
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ChunkIndex {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct Block {
    pub size: f32,
}

impl Block {
    pub fn new(size: f32) -> Self {
        Block { size }
    }
}

impl Default for Block {
    fn default() -> Self {
        Block { size: 1.0 }
    }
}

impl From<Block> for Mesh {
    fn from(block: Block) -> Self {
        shape::Box::new(block.size, block.size, block.size).into()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ChunkData {
    pub level: u32, // level or lod, normally 0
    pub index: ChunkIndex,
    pub voxels: [[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE], // row(z), col(x), depth(y)
}

impl ChunkData {
    pub fn new(chunk_index: ChunkIndex) -> Self {
        let perlin = Perlin::new(123);

        let chunk_offset = Vec3::new(
            chunk_index.x as f32 * CHUNK_SIZE as f32,
            chunk_index.y as f32 * CHUNK_SIZE as f32,
            chunk_index.z as f32 * CHUNK_SIZE as f32,
        );

        let mut voxels = [[[0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        (0..CHUNK_SIZE).for_each(|x| {
            (0..CHUNK_SIZE).for_each(|z| {
                let val = perlin.get([
                    (x as f64 + chunk_offset.x as f64) / WAVE_LENGTH as f64,
                    (z as f64 + chunk_offset.z as f64) / WAVE_LENGTH as f64,
                    0.0,
                ]);
                let land = 48.0.lerp(128.0, (val + 1.0) / 2.0) as i32;
                // println!(
                //     "Land at ({}, {}): {}",
                //     x + chunk_offset.x as usize,
                //     z + chunk_offset.z as usize,
                //     land
                // );
                (0..CHUNK_SIZE).for_each(|y: usize| {
                    if (y + chunk_offset.y as usize) as i32 > land {
                        voxels[x][y][z] = 0;
                    } else {
                        voxels[x][y][z] = 1;
                    }
                })
            })
        });

        ChunkData {
            level: 0,
            index: chunk_index,
            voxels,
        }
    }
}

impl Default for ChunkData {
    fn default() -> Self {
        ChunkData::new(ChunkIndex { x: 0, y: 0, z: 0 })
    }
}

struct CubeFace {
    cornor_indices: [u8; 4],     // cornor array index
    normal_index: FaceDirection, // +x:0 +y:1 +z:2 -x:3 -y:4 -z:5 same as FaceDirection
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum FaceDirection {
    Right = 0, // +x
    Top,       // +y
    Front,     // +z
    Left,      // -x
    Bottom,    // -y
    Back,      // -z
}

impl CubeFace {
    // coordinate as opengl, right hand coordinate, without any rotation
    // bottom-left is (0,0,0) top-right cornor is (1,1,1)
    // let 0,1,2,3, be the front face index, 4,5,6,7 be the back face index

    // front face triangles: 0,1,2,3 <0,1,2> <2,3,0>
    const FRONT_FACE: CubeFace = CubeFace {
        cornor_indices: [0, 1, 2, 3],
        normal_index: FaceDirection::Front,
    };

    // back face triangles: 4,5,6,7
    const BACK_FACE: CubeFace = CubeFace {
        cornor_indices: [4, 5, 6, 7],
        normal_index: FaceDirection::Back,
    };

    // left face triangles: 1,4,7,2
    const LEFT_FACE: CubeFace = CubeFace {
        cornor_indices: [1, 4, 7, 2],
        normal_index: FaceDirection::Left,
    };

    // right face triangles: 5,0,3,6
    const RIGHT_FACE: CubeFace = CubeFace {
        cornor_indices: [5, 0, 3, 6],
        normal_index: FaceDirection::Right,
    };

    // top face triangles: 5,4,1,0
    const TOP_FACE: CubeFace = CubeFace {
        cornor_indices: [5, 4, 1, 0],
        normal_index: FaceDirection::Top,
    };

    // bottom face triangles: 7,6,3,2
    const BOTTOM_FACE: CubeFace = CubeFace {
        cornor_indices: [7, 6, 3, 2],
        normal_index: FaceDirection::Bottom,
    };
}

#[derive(Debug, Clone)]
pub struct MeshData {
    positions: Vec<Vec3>,
    indices: Vec<u32>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
}

impl MeshData {
    fn new() -> Self {
        MeshData {
            positions: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
        }
    }
}

fn add_face(mesh: &mut MeshData, face: &CubeFace, offset: Vec3, size: Vec3) {
    let index_start: u32 = mesh.positions.len() as u32;

    for (i, &value) in face.cornor_indices.iter().enumerate() {
        mesh.positions.push(CORNORS[value as usize] * size + offset);
        mesh.normals.push(NORMALS[face.normal_index as usize]);
        // mesh.normals
        // .push((CORNORS[value as usize] - Vec3::new(0.5, 0.5, 0.5)).normalize()); // merge the normals of the same vertex
        mesh.uvs.push(UVS[i]);
    }

    mesh.indices.push(index_start);
    mesh.indices.push(index_start + 1);
    mesh.indices.push(index_start + 2);
    mesh.indices.push(index_start + 2);
    mesh.indices.push(index_start + 3);
    mesh.indices.push(index_start);
}

fn can_merge_mesh(voxel1: u8, voxel2: u8) -> bool {
    voxel1 == voxel2
}

fn default_mesh(chunk: ChunkData) -> MeshData {
    let mut mesh_data = MeshData::new();
    (0..CHUNK_SIZE).for_each(|y| {
        (0..CHUNK_SIZE).for_each(|z| {
            (0..CHUNK_SIZE).for_each(|x| {
                // println!("Element at ({}, {}, {}): {}", x, y, z, elem);
                if chunk.voxels[x][y][z] == 0 {
                    return;
                }

                let offset = Vec3::new(
                    chunk.index.x as f32 * CHUNK_SIZE as f32,
                    chunk.index.y as f32 * CHUNK_SIZE as f32,
                    chunk.index.z as f32 * CHUNK_SIZE as f32,
                ) + Vec3::new(x as f32, y as f32, z as f32);

                if y == CHUNK_SIZE - 1 || (y < CHUNK_SIZE - 1 && chunk.voxels[x][y + 1][z] == 0) {
                    add_face(&mut mesh_data, &CubeFace::TOP_FACE, offset, Vec3::ONE);
                }

                if y == 0 || (y > 0 && chunk.voxels[x][y - 1][z] == 0) {
                    add_face(&mut mesh_data, &CubeFace::BOTTOM_FACE, offset, Vec3::ONE);
                }

                if x == 0 || (x > 0 && chunk.voxels[x - 1][y][z] == 0) {
                    add_face(&mut mesh_data, &CubeFace::LEFT_FACE, offset, Vec3::ONE);
                }

                if x == CHUNK_SIZE - 1 || (x < CHUNK_SIZE - 1 && chunk.voxels[x + 1][y][z] == 0) {
                    add_face(&mut mesh_data, &CubeFace::RIGHT_FACE, offset, Vec3::ONE);
                }

                if z == CHUNK_SIZE - 1 || (z < CHUNK_SIZE - 1 && chunk.voxels[x][y][z + 1] == 0) {
                    add_face(&mut mesh_data, &CubeFace::FRONT_FACE, offset, Vec3::ONE);
                }

                if z == 0 || (z > 0 && chunk.voxels[x][y][z - 1] == 0) {
                    add_face(&mut mesh_data, &CubeFace::BACK_FACE, offset, Vec3::ONE);
                }
            })
        })
    });

    mesh_data
}

pub fn greedy_meshing(chunk: &ChunkData) -> MeshData {
    let mut sizes: [[[Vec3; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE] =
        [[[Vec3::ONE; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
    (0..CHUNK_SIZE).for_each(|y| {
        (0..CHUNK_SIZE).for_each(|z| {
            (1..CHUNK_SIZE).for_each(|x| {
                if can_merge_mesh(chunk.voxels[x][y][z], chunk.voxels[x - 1][y][z]) {
                    sizes[x][y][z].x += sizes[x - 1][y][z].x;
                    sizes[x - 1][y][z] = Vec3::ZERO;
                }
            })
        })
    });

    (0..CHUNK_SIZE).for_each(|y| {
        (0..CHUNK_SIZE).for_each(|x| {
            (1..CHUNK_SIZE).for_each(|z| {
                if sizes[x][y][z] == Vec3::ZERO || sizes[x][y][z - 1] == Vec3::ZERO {
                    return;
                }
                if can_merge_mesh(chunk.voxels[x][y][z], chunk.voxels[x][y][z - 1])
                    && sizes[x][y][z - 1].x == sizes[x][y][z].x
                {
                    sizes[x][y][z].z += sizes[x][y][z - 1].z;
                    sizes[x][y][z - 1] = Vec3::ZERO;
                }
            })
        })
    });

    (0..CHUNK_SIZE).for_each(|x| {
        (0..CHUNK_SIZE).for_each(|z| {
            (1..CHUNK_SIZE).for_each(|y| {
                if sizes[x][y][z] == Vec3::ZERO || sizes[x][y - 1][z] == Vec3::ZERO {
                    return;
                }
                if can_merge_mesh(chunk.voxels[x][y][z], chunk.voxels[x][y - 1][z])
                    && sizes[x][y - 1][z].x == sizes[x][y][z].x
                    && sizes[x][y - 1][z].z == sizes[x][y][z].z
                {
                    sizes[x][y][z].y += sizes[x][y - 1][z].y;
                    sizes[x][y - 1][z] = Vec3::ZERO;
                }
            })
        })
    });

    let mut mesh_data = MeshData::new();
    (0..CHUNK_SIZE).for_each(|y| {
        (0..CHUNK_SIZE).for_each(|z| {
            (0..CHUNK_SIZE).for_each(|x| {
                // println!("Element at ({}, {}, {}): {}", x, y, z, elem);
                if chunk.voxels[x][y][z] == 0 {
                    return;
                }

                if sizes[x][y][z] == Vec3::ZERO {
                    return;
                }

                let offset = Vec3::new(
                    chunk.index.x as f32 * CHUNK_SIZE as f32,
                    chunk.index.y as f32 * CHUNK_SIZE as f32,
                    chunk.index.z as f32 * CHUNK_SIZE as f32,
                ) + Vec3::new(x as f32, y as f32, z as f32);

                // top face of the chunk
                if y == CHUNK_SIZE - 1 {
                    add_face(
                        &mut mesh_data,
                        &CubeFace::TOP_FACE,
                        offset
                            + Vec3::new(-(sizes[x][y][z].x - 1.0), 0.0, -(sizes[x][y][z].z - 1.0)),
                        Vec3::new(sizes[x][y][z].x, 1.0, sizes[x][y][z].z),
                    );
                }

                if y < CHUNK_SIZE - 1
                // && sizes[x][y + 1][z].x >= sizes[x][y][z].x  // can't simple check the cell above, because the cell above may be merged with other cells
                // && sizes[x][y + 1][z].z >= sizes[x][y][z].z)
                {
                    // check if the top surface is exposed
                    // if not, skip the top face
                    let mut is_exposed = false;
                    'check_surface: for z1 in (1 + z - sizes[x][y][z].z as usize)..=z {
                        for x1 in (1 + x - sizes[x][y][z].x as usize)..=x {
                            if chunk.voxels[x1][y + 1][z1] == 0 {
                                is_exposed = true;
                                break 'check_surface;
                            }
                        }
                    }
                    if is_exposed {
                        add_face(
                            &mut mesh_data,
                            &CubeFace::TOP_FACE,
                            offset
                                + Vec3::new(
                                    -(sizes[x][y][z].x - 1.0),
                                    0.0,
                                    -(sizes[x][y][z].z - 1.0),
                                ),
                            Vec3::new(sizes[x][y][z].x, 1.0, sizes[x][y][z].z),
                        );
                    }
                }

                // bottom face of the chunk
                if 1 + y - sizes[x][y][z].y as usize == 0 {
                    add_face(
                        &mut mesh_data,
                        &CubeFace::BOTTOM_FACE,
                        offset
                            + Vec3::new(-(sizes[x][y][z].x - 1.0), 0.0, -(sizes[x][y][z].z - 1.0))
                            + Vec3::new(0.0, -(sizes[x][y][z].y - 1.0), 0.0), // because after merge, the cell has a size of non-zero is the top-right front cell
                        Vec3::new(sizes[x][y][z].x, 1.0, sizes[x][y][z].z),
                    );
                } else {
                    // check if the bottom surface is exposed
                    // if not, skip the bottom face
                    let mut is_exposed = false;
                    'check_surface: for z1 in (1 + z - sizes[x][y][z].z as usize)..=z {
                        for x1 in (1 + x - sizes[x][y][z].x as usize)..=x {
                            if chunk.voxels[x1][y - sizes[x][y][z].y as usize][z1] == 0 {
                                is_exposed = true;
                                break 'check_surface;
                            }
                        }
                    }

                    if is_exposed {
                        add_face(
                            &mut mesh_data,
                            &CubeFace::BOTTOM_FACE,
                            offset
                                + Vec3::new(
                                    -(sizes[x][y][z].x - 1.0),
                                    0.0,
                                    -(sizes[x][y][z].z - 1.0),
                                )
                                + Vec3::new(0.0, -(sizes[x][y][z].y - 1.0), 0.0), // because after merge, the cell has a size of non-zero is the top-right front cell
                            Vec3::new(sizes[x][y][z].x, 1.0, sizes[x][y][z].z),
                        );
                    }
                }

                // left face of the chunk
                if 1 + x - sizes[x][y][z].x as usize == 0 {
                    add_face(
                        &mut mesh_data,
                        &CubeFace::LEFT_FACE,
                        offset
                            + Vec3::new(0.0, -(sizes[x][y][z].y - 1.0), -(sizes[x][y][z].z - 1.0))
                            + Vec3::new(-(sizes[x][y][z].x - 1.0), 0.0, 0.0),
                        Vec3::new(1.0, sizes[x][y][z].y, sizes[x][y][z].z),
                    );
                } else {
                    // check if the left surface is exposed
                    // if not, skip the left face
                    let mut is_exposed = false;
                    'check_surface: for z1 in (1 + z - sizes[x][y][z].z as usize)..=z {
                        for y1 in (1 + y - sizes[x][y][z].y as usize)..=y {
                            if chunk.voxels[x - sizes[x][y][z].x as usize][y1][z1] == 0 {
                                is_exposed = true;
                                break 'check_surface;
                            }
                        }
                    }

                    if is_exposed {
                        add_face(
                            &mut mesh_data,
                            &CubeFace::LEFT_FACE,
                            offset
                                + Vec3::new(
                                    0.0,
                                    -(sizes[x][y][z].y - 1.0),
                                    -(sizes[x][y][z].z - 1.0),
                                )
                                + Vec3::new(-(sizes[x][y][z].x - 1.0), 0.0, 0.0),
                            Vec3::new(1.0, sizes[x][y][z].y, sizes[x][y][z].z),
                        );
                    }
                }

                // right face of the chunk
                if x == CHUNK_SIZE - 1 {
                    add_face(
                        &mut mesh_data,
                        &CubeFace::RIGHT_FACE,
                        offset
                            + Vec3::new(0.0, -(sizes[x][y][z].y - 1.0), -(sizes[x][y][z].z - 1.0)),
                        Vec3::new(1.0, sizes[x][y][z].y, sizes[x][y][z].z),
                    );
                } else {
                    // check if the right surface is exposed
                    // if not, skip the right face
                    let mut is_exposed = false;
                    'check_surface: for z1 in (1 + z - sizes[x][y][z].z as usize)..=z {
                        for y1 in (1 + y - sizes[x][y][z].y as usize)..=y {
                            if chunk.voxels[x + 1][y1][z1] == 0 {
                                is_exposed = true;
                                break 'check_surface;
                            }
                        }
                    }

                    if is_exposed {
                        add_face(
                            &mut mesh_data,
                            &CubeFace::RIGHT_FACE,
                            offset
                                + Vec3::new(
                                    0.0,
                                    -(sizes[x][y][z].y - 1.0),
                                    -(sizes[x][y][z].z - 1.0),
                                ),
                            Vec3::new(1.0, sizes[x][y][z].y, sizes[x][y][z].z),
                        );
                    }
                }

                // front face of the chunk
                if z == CHUNK_SIZE - 1 {
                    add_face(
                        &mut mesh_data,
                        &CubeFace::FRONT_FACE,
                        offset
                            + Vec3::new(-(sizes[x][y][z].x - 1.0), -(sizes[x][y][z].y - 1.0), 0.0),
                        Vec3::new(sizes[x][y][z].x, sizes[x][y][z].y, 1.0),
                    );
                } else {
                    // check if the front surface is exposed
                    // if not, skip the front face
                    let mut is_exposed = false;
                    'check_surface: for x1 in (1 + x - sizes[x][y][z].x as usize)..=x {
                        for y1 in (1 + y - sizes[x][y][z].y as usize)..=y {
                            if chunk.voxels[x1][y1][z + 1] == 0 {
                                is_exposed = true;
                                break 'check_surface;
                            }
                        }
                    }

                    if is_exposed {
                        add_face(
                            &mut mesh_data,
                            &CubeFace::FRONT_FACE,
                            offset
                                + Vec3::new(
                                    -(sizes[x][y][z].x - 1.0),
                                    -(sizes[x][y][z].y - 1.0),
                                    0.0,
                                ),
                            Vec3::new(sizes[x][y][z].x, sizes[x][y][z].y, 1.0),
                        );
                    }
                }

                // back face of the chunk
                if 1 + z - sizes[x][y][z].z as usize == 0 {
                    add_face(
                        &mut mesh_data,
                        &CubeFace::BACK_FACE,
                        offset
                            + Vec3::new(-(sizes[x][y][z].x - 1.0), -(sizes[x][y][z].y - 1.0), 0.0)
                            + Vec3::new(0.0, 0.0, -(sizes[x][y][z].z - 1.0)),
                        Vec3::new(sizes[x][y][z].x, sizes[x][y][z].y, 1.0),
                    );
                } else {
                    // check if the back surface is exposed
                    // if not, skip the back face
                    let mut is_exposed = false;
                    'check_surface: for x1 in (1 + x - sizes[x][y][z].x as usize)..=x {
                        for y1 in (1 + y - sizes[x][y][z].y as usize)..=y {
                            if chunk.voxels[x1][y1][z - sizes[x][y][z].z as usize] == 0 {
                                is_exposed = true;
                                break 'check_surface;
                            }
                        }
                    }

                    if is_exposed {
                        add_face(
                            &mut mesh_data,
                            &CubeFace::BACK_FACE,
                            offset
                                + Vec3::new(
                                    -(sizes[x][y][z].x - 1.0),
                                    -(sizes[x][y][z].y - 1.0),
                                    0.0,
                                )
                                + Vec3::new(0.0, 0.0, -(sizes[x][y][z].z - 1.0)),
                            Vec3::new(sizes[x][y][z].x, sizes[x][y][z].y, 1.0),
                        );
                    }
                }
            })
        })
    });

    mesh_data
}

impl From<MeshData> for Mesh {
    fn from(value: MeshData) -> Self {
        // let mesh_data = merge_vertex(&mesh_data, 0.01);
        let indices = Indices::U32(value.indices);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, value.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, value.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, value.uvs);
        mesh
    }
}

fn merge_vertex(source: &MeshData, threshold: f32) -> MeshData {
    let mut dest = MeshData::new();
    let mut vertex_map = HashMap::new();
    (0..source.positions.len()).for_each(|i| {
        let mut found = false;
        for j in 0..dest.positions.len() {
            if Vec3::length_squared(source.positions[i] - dest.positions[j]) < threshold {
                vertex_map.insert(i, j);
                found = true;
                break;
            }
        }

        if !found {
            vertex_map.insert(i, dest.positions.len());
            dest.positions.push(source.positions[i]);
            dest.normals.push(source.normals[i]);
        }
    });

    (0..source.indices.len()).for_each(|i| {
        dest.indices
            .push(vertex_map[&(source.indices[i] as usize)] as u32);
    });
    // dest.normals = source.normals.clone();
    dest
}

/// Combine multiple meshes into one mesh
pub fn combine_meshes(meshes: &[MeshData]) -> MeshData {
    let mut mesh_data = MeshData::new();
    let mut index_start: u32 = 0;
    for mesh in meshes {
        mesh_data.positions.extend(mesh.positions.iter());
        mesh_data.normals.extend(mesh.normals.iter());
        mesh_data.uvs.extend(mesh.uvs.iter());
        mesh_data
            .indices
            .extend(mesh.indices.iter().map(|i| i + index_start));
        index_start += mesh.positions.len() as u32;
    }
    mesh_data
}

#[derive(Component)]
pub struct ChunkMesh {
    pub dirty: bool,
    pub mesh: Handle<Mesh>,
    pub merged: bool,
}

#[derive(Resource, Default)]
pub struct VoxelData {
    pub chunks: HashMap<ChunkIndex, ChunkData>,
}

#[derive(Resource, Default)]
pub struct VoxelMeshes {
    pub columns: HashMap<ChunkColumn, Entity>,
}

#[derive(Component)]
pub struct Chunk {
    pub index: ChunkIndex,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkColumn {
    pub x: i32,
    pub z: i32,
}

#[derive(Component)]
pub struct ColumnMesh {
    pub column: ChunkColumn,
    pub dirty: bool,
    pub mesh: Handle<Mesh>,
}

pub fn get_chunk_index(pos: &Vec3) -> ChunkIndex {
    ChunkIndex {
        x: (pos.x / CHUNK_SIZE as f32).floor() as i32,
        y: (pos.y / CHUNK_SIZE as f32).floor() as i32,
        z: (pos.z / CHUNK_SIZE as f32).floor() as i32,
    }
}

#[derive(Resource, Default)]
pub struct ChunkMeshesUpdateQueue {
    pub queue: HashSet<ChunkColumn>,
}

#[derive(Resource, Default)]
pub struct VoxelModifyQueue {
    pub queue: Vec<(Vec3, u8)>,
}

fn to_voxel_position(pos: &Vec3) -> Vec3 {
    Vec3 {
        x: pos.x.floor(),
        y: pos.y.floor(),
        z: pos.z.floor(),
    }
}

pub fn get_intersected_voxels(start_point: &Vec3, direction: &Vec3, range: f32) -> Vec<Vec3> {
    // Ensures passed direction is normalized
    let n_direction = direction.normalize();
    let end_point = *start_point + n_direction * range;
    let start_voxel = to_voxel_position(start_point);

    // +1, -1, or 0
    let step_x = if n_direction.x > 0.0 {
        1.0
    } else if n_direction.x < 0.0 {
        -1.0
    } else {
        0.0
    };
    let step_y = if n_direction.y > 0.0 {
        1.0
    } else if n_direction.y < 0.0 {
        -1.0
    } else {
        0.0
    };
    let step_z = if n_direction.z > 0.0 {
        1.0
    } else if n_direction.z < 0.0 {
        -1.0
    } else {
        0.0
    };

    let t_delta_x = if step_x != 0.0 {
        f32::min(step_x / (end_point.x - start_point.x), f32::MAX)
    } else {
        f32::MAX
    };

    let t_delta_y = if step_y != 0.0 {
        f32::min(step_y / (end_point.y - start_point.y), f32::MAX)
    } else {
        f32::MAX
    };

    let t_delta_z = if step_z != 0.0 {
        f32::min(step_z / (end_point.z - start_point.z), f32::MAX)
    } else {
        f32::MAX
    };

    let mut t_max_x = if step_x > 0.0 {
        t_delta_x * (1.0 - start_point.x + start_voxel.x)
    } else {
        t_delta_x * (start_point.x - start_voxel.x)
    };

    let mut t_max_y = if step_y > 0.0 {
        t_delta_y * (1.0 - start_point.y + start_voxel.y)
    } else {
        t_delta_y * (start_point.y - start_voxel.y)
    };

    let mut t_max_z = if step_z > 0.0 {
        t_delta_z * (1.0 - start_point.z + start_voxel.z)
    } else {
        t_delta_z * (start_point.z - start_voxel.z)
    };

    let mut current_voxel = start_voxel;
    let mut intersected = Vec::new();
    intersected.push(start_voxel);

    // sanity check to prevent leak
    while intersected.len() < range as usize * 3 {
        if (t_max_x < t_max_y) {
            if (t_max_x < t_max_z) {
                current_voxel.x += step_x;
                t_max_x += t_delta_x;
            } else {
                current_voxel.z += step_z;
                t_max_z += t_delta_z;
            }
        } else {
            if (t_max_y < t_max_z) {
                current_voxel.y += step_y;
                t_max_y += t_delta_y;
            } else {
                current_voxel.z += step_z;
                t_max_z += t_delta_z;
            }
        }
        if (t_max_x > 1.0 && t_max_y > 1.0 && t_max_z > 1.0) {
            break;
        }
        intersected.push(current_voxel);
    }
    intersected
}

pub struct VoxelLocalIndex {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

pub fn pos_to_voxel(pos: &Vec3) -> (ChunkIndex, VoxelLocalIndex) {
    let chunk_index = get_chunk_index(pos);
    (
        chunk_index,
        VoxelLocalIndex {
            x: (pos.x - chunk_index.x as f32 * CHUNK_SIZE as f32).floor() as u8,
            y: (pos.y - chunk_index.y as f32 * CHUNK_SIZE as f32).floor() as u8,
            z: (pos.z - chunk_index.z as f32 * CHUNK_SIZE as f32).floor() as u8,
        },
    )
}

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct VoxelSettings {
    pub sight_range: u8, // in chunk
    pub interact_distance: f32,
}
