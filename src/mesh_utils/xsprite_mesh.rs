use crate::{
    blocks::Block,
    chunk::{XSpriteVIVI, CHUNK_TOTAL_BLOCKS},
    prelude::*,
};

pub struct XSpriteMetaData {
    pub vivi: XSpriteVIVI,
    pub log: Vec<(VoxelChange, Block, usize)>,
}

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

    (
        mesh,
        XSpriteMetaData {
            vivi: data_structure.to_vec(),
            log: vec![],
        },
    )
}

pub fn update_xsprite_mesh(
    reg: &impl VoxelRegistry<Voxel = Block>,
    mesh: &mut Mesh,
    md: &mut XSpriteMetaData,
) {
    for (change, block, index) in md.log.iter() {
        match change {
            VoxelChange::Added => {
                add_xsprite_voxel(mesh, &mut md.vivi, *index, reg.get_mesh(block).unwrap())
            }
            VoxelChange::Broken => {
                println!("{}", *index);
                remove_xsprite_voxel(mesh, &mut md.vivi, *index);
            }
            _ => debug_assert!(false, "tried using unsupported VoxelChange in XSpriteMesh"),
        }
    }
    md.log.clear();
}

fn remove_xsprite_voxel(mesh: &mut Mesh, md: &mut XSpriteVIVI, index: usize) {
    let (vertex_start, vertex_end, index_start, index_end) = md[index];
    let voffset = vertex_end - vertex_start;
    let ioffset = index_end - index_start;
    let mut i = 0;
    for (_, vav) in mesh.attributes_mut() {
        i = 0;
        for vertex in vertex_start..vertex_end {
            vav.remove(vertex);
            i += 1;
        }
    }
    println!("{i}");
    // FIX BUG IN THE NEXT 10 LINES
    if let Some(Indices::U32(ref mut indices)) = mesh.indices_mut() {
        for i in index_start..index_end {
            indices.remove(i as usize);
        }
        for i in 0..indices.len() {
            if indices[i] as usize >= vertex_end {
                indices[i] -= voffset as u32;
            }
        }
    }
    for i in 0..md.len() {
        if md[i].0 >= vertex_end {
            md[i].0 -= voffset;
            md[i].1 -= voffset;
            md[i].2 -= ioffset as u32;
            md[i].3 -= ioffset as u32;
        }
    }
    md[index] = (0, 0, 0, 0)
}

fn add_xsprite_voxel<'a>(
    mesh: &mut Mesh,
    md: &'a mut XSpriteVIVI,
    index: usize,
    voxel_mesh: &Mesh,
) {
    let ver_count = mesh.count_vertices();
    for (id, vav) in mesh.attributes_mut() {
        let vav2 = voxel_mesh.attribute(id).unwrap();
        vav.extend(vav2);
    }
    let ver_count2 = mesh.count_vertices();
    let mut ind_count = 0;
    let mut ind_count2 = 0;
    if let Some(Indices::U32(ref mut indices)) = mesh.indices_mut() {
        ind_count = indices.len();
        if let Some(Indices::U32(voxel_indices)) = voxel_mesh.indices() {
            let indices_offset: Vec<u32> = voxel_indices
                .clone()
                .iter()
                .map(|x| *x + ver_count as u32)
                .collect();
            indices.extend(indices_offset);
            ind_count2 = indices.len();
        }
    }
    md[index] = (ver_count, ver_count2, ind_count as u32, ind_count2 as u32);
}
