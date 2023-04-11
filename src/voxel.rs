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

struct MeshFace {
    vertices: [u8; 12],
    normal: u8, // +x:0 +y:1 +z:2 -x:3 -y:4 -z:5
}

impl MeshFace {
    const FRONT_FACE: MeshFace = MeshFace {
        vertices: [1, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1],
        normal: 2,
    };
    const LEFT_FACE: MeshFace = MeshFace {
        vertices: [0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1],
        normal: 3,
    };
    const BACK_FACE: MeshFace = MeshFace {
        vertices: [0, 1, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0],
        normal: 5,
    };
    const RIGHT_FACE: MeshFace = MeshFace {
        vertices: [1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 0],
        normal: 0,
    };

    const TOP_FACE: MeshFace = MeshFace {
        vertices: [1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1],
        normal: 1,
    };

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

fn add_face(mesh: &mut ChunkMesh, face: &MeshFace, voxel_index: &VoxelIndex, texture: tex_t) {
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
