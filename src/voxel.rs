use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

#[allow(non_camel_case_types)]
type vertex_t = u32;

#[allow(non_camel_case_types)]
type index_t = u32;

#[allow(non_camel_case_types)]
type tex_t = u32;

struct ChunkMesh {
    vertices: Vec<vertex_t>,
    indices: Vec<index_t>,
}

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

const NORMALS: [Vec3; 6] = [
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(0.0, 0.0, 1.0),
    Vec3::new(-1.0, 0.0, 0.0),
    Vec3::new(0.0, -1.0, 0.0),
    Vec3::new(0.0, 0.0, -1.0),
];

struct MeshFace {
    vertices: [u8; 12], // 4 vertices, each vertex has 3 coordinates, order: x,y,z
    normal: u8,         // +x:0 +y:1 +z:2 -x:3 -y:4 -z:5
}

impl MeshFace {
    // coordinate as opengl, right hand coordinate, without any rotation
    // bottom-left is (0,0,0) top-right cornor is (1,1,1)
    // let 1,2,3,4 be the front face index, 5,6,7,8 be the back face index

    // front face triangles: 1,2,3,4 <1,2,3> <3,4,1>
    const FRONT_FACE: MeshFace = MeshFace {
        vertices: [1, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1],
        normal: 2,
    };

    // back face triangles: 5,6,7,8 <5,6,7> <7,8,5>
    const BACK_FACE: MeshFace = MeshFace {
        vertices: [0, 1, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0],
        normal: 5,
    };

    // left face triangles: 2,5,8,3 <2,5,8> <8,3,2>
    const LEFT_FACE: MeshFace = MeshFace {
        vertices: [0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1],
        normal: 3,
    };

    // right face triangles: 6,1,4,7 <6,1,4> <4,7,6>
    const RIGHT_FACE: MeshFace = MeshFace {
        vertices: [1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 0],
        normal: 0,
    };

    // top face triangles: 6,5,2,1 <6,5,2> <2,1,6>
    const TOP_FACE: MeshFace = MeshFace {
        vertices: [1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1],
        normal: 1,
    };

    // bottom face triangles: 8,7,4,3 <8,7,4> <4,3,8>
    const BOTTOM_FACE: MeshFace = MeshFace {
        vertices: [0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1],
        normal: 4,
    };
}

struct VoxelIndex {
    x: u8,
    y: u8,
    z: u8,
}

fn add_compressed_face(
    mesh: &mut ChunkMesh,
    face: &MeshFace,
    voxel_index: &VoxelIndex,
    texture: tex_t,
) {
    for i in 0..4 {
        let x: u32 = (face.vertices[i * 3] + voxel_index.x) as u32;
        let y: u32 = (face.vertices[i * 3 + 1] + voxel_index.y) as u32;
        let z: u32 = (face.vertices[i * 3 + 2] + voxel_index.z) as u32;

        let vertex: u32 =
            x | y << 4 | z << 8 | (face.normal as u32) << 12 | (i as u32) << 15 | texture << 17;

        mesh.vertices.push(vertex);
    }

    let index_start: u32 = (mesh.vertices.len() - 4) as u32;
    mesh.indices.push(index_start);
    mesh.indices.push(index_start + 1);
    mesh.indices.push(index_start + 2);
    mesh.indices.push(index_start + 2);
    mesh.indices.push(index_start + 3);
    mesh.indices.push(index_start);
}

#[derive(Debug, Copy, Clone)]
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
    pub voxels: [[[u8; 16]; 16]; 16], // row(z), col(x), depth(y)
}

impl ChunkData {
    pub fn new() -> Self {
        ChunkData {
            level: 0,
            index: ChunkIndex { x: 0, y: 0, z: 0 },
            voxels: [[[0; 16]; 16]; 16],
        }
    }
}

impl Default for ChunkData {
    fn default() -> Self {
        ChunkData::new()
    }
}

impl From<ChunkData> for Mesh {
    fn from(chunk: ChunkData) -> Self {
        let land_depth = 0;
        let mut meshData = MeshData::new();
        for (y, depth) in chunk.voxels.iter().enumerate() {
            if y == land_depth {
                for (x, col) in depth.iter().enumerate() {
                    for (z, &elem) in col.iter().enumerate() {
                        println!("Element at ({}, {}, {}): {}", x, y, z, elem);
                        add_face(
                            &mut meshData,
                            &CubeFace::TOP_FACE,
                            Vec3::new(x as f32, y as f32, z as f32),
                        );
                    }
                }
            }
        }

        let indices = Indices::U32(meshData.indices);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, meshData.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, meshData.normals);
        // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, meshData.uvs);
        mesh
    }
}

struct CubeFace {
    cornor_indices: [u8; 4], // cornor array index
    normal_index: u8,
}

impl CubeFace {
    // coordinate as opengl, right hand coordinate, without any rotation
    // bottom-left is (0,0,0) top-right cornor is (1,1,1)
    // let 0,1,2,3, be the front face index, 4,5,6,7 be the back face index

    // front face triangles: 0,1,2,3 <0,1,2> <2,3,0>
    const FRONT_FACE: CubeFace = CubeFace {
        cornor_indices: [0, 1, 2, 3],
        normal_index: 2,
    };

    // back face triangles: 4,5,6,7
    const BACK_FACE: CubeFace = CubeFace {
        cornor_indices: [4, 5, 6, 7],
        normal_index: 5,
    };

    // left face triangles: 1,4,7,2
    const LEFT_FACE: CubeFace = CubeFace {
        cornor_indices: [1, 4, 7, 2],
        normal_index: 3,
    };

    // right face triangles: 5,0,3,6
    const RIGHT_FACE: CubeFace = CubeFace {
        cornor_indices: [5, 0, 3, 6],
        normal_index: 0,
    };

    // top face triangles: 5,4,1,0
    const TOP_FACE: CubeFace = CubeFace {
        cornor_indices: [5, 4, 1, 0],
        normal_index: 1,
    };

    // bottom face triangles: 7,6,3,2
    const BOTTOM_FACE: CubeFace = CubeFace {
        cornor_indices: [7, 6, 3, 2],
        normal_index: 4,
    };
}

struct MeshData {
    positions: Vec<Vec3>,
    indices: Vec<u32>,
    normals: Vec<Vec3>,
}

impl MeshData {
    fn new() -> Self {
        MeshData {
            positions: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
        }
    }
}

fn add_face(mesh: &mut MeshData, face: &CubeFace, offset: Vec3) {
    let index_start: u32 = mesh.positions.len() as u32;

    for (_, &value) in face.cornor_indices.iter().enumerate() {
        mesh.positions.push(CORNORS[value as usize] + offset);
        mesh.normals.push(NORMALS[face.normal_index as usize]);
    }

    mesh.indices.push(index_start);
    mesh.indices.push(index_start + 1);
    mesh.indices.push(index_start + 2);
    mesh.indices.push(index_start + 2);
    mesh.indices.push(index_start + 3);
    mesh.indices.push(index_start);
}
