use crate::{
    blocks::Block,
    chunk::{XSpriteMetaData, CHUNK_TOTAL_BLOCKS},
    prelude::*,
};

pub fn meshify_xsprite_voxels(
    reg: &impl VoxelRegistry<Voxel = Block>,
    grid: &[Block],
    dims: Dimensions,
) -> (Mesh, XSpriteMetaData) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut indices: Vec<u32> = vec![];
    let mut colors: Vec<[f32; 4]> = vec![];
    let mut positions: Vec<[f32; 3]> = vec![];
    let mut uvs: Vec<[f32; 2]> = vec![];
    let mut normals: Vec<[f32; 3]> = vec![];

    // data structure similar to VIVI, to map voxel index
    let mut data_structure = [(usize::MAX, usize::MAX, u32::MAX, u32::MAX); CHUNK_TOTAL_BLOCKS];

    let width = dims.0;
    let length = dims.2;
    let height = dims.1;

    let voxel_dims = reg.get_voxel_dimensions();

    for k in 0..height {
        for j in 0..length {
            for i in 0..width {
                let cord = k * length * width + j * width + i;
                let block = grid[cord];
                let position_offset = (
                    i as f32 * voxel_dims[0],
                    k as f32 * voxel_dims[1],
                    j as f32 * voxel_dims[2],
                );

                if let VoxelMesh::CustomMesh(custom_mesh) = reg.get_mesh(&block) {
                    let pos_attribute = custom_mesh
                        .attribute(Mesh::ATTRIBUTE_POSITION)
                        .expect("couldn't get voxel mesh data");
                    let VertexAttributeValues::Float32x3(pos) = pos_attribute else {
                        panic!(
                            "Unexpected vertex format for position attribute, expected Float32x3."
                        );
                    };
                    let pos: Vec<[f32; 3]> = pos
                        .iter()
                        .map(|[x, y, z]| {
                            [
                                *x + position_offset.0,
                                *y + position_offset.1,
                                *z + position_offset.2,
                            ]
                        })
                        .collect();

                    let VertexAttributeValues::Float32x4(col) = custom_mesh
                        .attribute(Mesh::ATTRIBUTE_COLOR)
                        .expect("couldn't get mesh data")
                    else {
                        panic!("Incorrect format for colors");
                    };
                    let VertexAttributeValues::Float32x2(uv) = custom_mesh
                        .attribute(Mesh::ATTRIBUTE_UV_0)
                        .expect("couldn't get mesh data")
                    else {
                        panic!("Incorrect format for uvs");
                    };
                    let VertexAttributeValues::Float32x3(nor) = custom_mesh
                        .attribute(Mesh::ATTRIBUTE_NORMAL)
                        .expect("couldn't get mesh data")
                    else {
                        panic!("Incorrect format for normals");
                    };
                    let Indices::U32(ind) =
                        custom_mesh.indices().expect("couldn't get indices data")
                    else {
                        panic!("Expected U32 indices format");
                    };
                    let ind: Vec<u32> = ind.iter().map(|i| *i + positions.len() as u32).collect();

                    data_structure[cord].0 = positions.len();
                    data_structure[cord].2 = indices.len() as u32;

                    positions.extend(pos);
                    colors.extend(col);
                    normals.extend(nor);
                    uvs.extend(uv);
                    indices.extend(ind);

                    data_structure[cord].1 = positions.len();
                    data_structure[cord].3 = indices.len() as u32;
                }
            }
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.set_indices(Some(Indices::U32(indices)));

    (mesh, Box::new(data_structure))
}
