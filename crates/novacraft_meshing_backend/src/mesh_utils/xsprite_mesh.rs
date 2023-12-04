use crate::prelude::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy::render::render_resource::PrimitiveTopology;

pub type XSpriteVIVI = Vec<(usize, usize, u32, u32)>;

pub struct XSpriteMetaData<T> {
    pub vivi: XSpriteVIVI,
    pub log: Vec<(VoxelChange, T, BlockPos)>,
}

pub fn meshify_xsprite_voxels<T: Copy, const N: usize>(
    reg: &impl VoxelRegistry<Voxel = T>,
    grid: &ChunkGrid<T, N>,
) -> (Mesh, XSpriteMetaData<T>) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut indices: Vec<u32> = vec![];
    let mut colors: Vec<[f32; 4]> = vec![];
    let mut positions: Vec<[f32; 3]> = vec![];
    let mut uvs: Vec<[f32; 2]> = vec![];
    let mut normals: Vec<[f32; 3]> = vec![];

    // data structure similar to VIVI, to map voxel index
    let mut data_structure = vec![(usize::MIN, usize::MIN, u32::MIN, u32::MIN); grid.len()];

    let width = grid.dims.x;
    let length = grid.dims.z;
    let height = grid.dims.y;

    let voxel_dims = reg.get_voxel_dimensions();

    let mut block_pos = BlockPos::new(0, 0, 0);
    for k in 0..height {
        block_pos.y = k;
        for j in 0..length {
            block_pos.z = j;
            for i in 0..width {
                block_pos.x = i;
                let block = grid.get_block(block_pos).unwrap();
                let position_offset = (
                    i as f32 * voxel_dims[0],
                    k as f32 * voxel_dims[1],
                    j as f32 * voxel_dims[2],
                );

                if let VoxelMesh::XSprite(custom_mesh) = reg.get_mesh(&block) {
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

                    let block_index = pos_to_index(block_pos, grid.dims).unwrap();
                    data_structure[block_index].0 = positions.len();
                    data_structure[block_index].2 = indices.len() as u32;

                    positions.extend(pos);
                    colors.extend(col);
                    normals.extend(nor);
                    uvs.extend(uv);
                    indices.extend(ind);

                    data_structure[block_index].1 = positions.len();
                    data_structure[block_index].3 = indices.len() as u32;
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

pub fn update_xsprite_mesh<T>(
    reg: &impl VoxelRegistry<Voxel = T>,
    mesh: &mut Mesh,
    md: &mut XSpriteMetaData<T>,
    dims: Dimensions,
) {
    for (change, block, block_pos) in md.log.iter() {
        let block_index: BlockIndex = pos_to_index(*block_pos, dims).unwrap();
        match change {
            VoxelChange::Added => add_xsprite_voxel(
                mesh,
                &mut md.vivi,
                *block_pos,
                reg.get_mesh(block).unwrap(),
                reg.get_voxel_dimensions().into(),
                dims,
            ),
            VoxelChange::Broken => {
                remove_xsprite_voxel(mesh, &mut md.vivi, block_index);
            }
            _ => debug_assert!(false, "tried using unsupported VoxelChange in XSpriteMesh"),
        }
    }
    md.log.clear();
}

fn remove_xsprite_voxel(mesh: &mut Mesh, md: &mut XSpriteVIVI, block_index: BlockIndex) {
    let (vertex_start, vertex_end, index_start, index_end) = md[block_index];
    let last = vertex_end == mesh.count_vertices();
    if vertex_end - vertex_start > 0 {
        for (_, vav) in mesh.attributes_mut() {
            for vertex in (vertex_start..vertex_end).rev() {
                vav.swap_remove(vertex);
            }
        }
        if let Some(Indices::U32(ref mut indices)) = mesh.indices_mut() {
            for _ in (index_start..index_end).rev() {
                // indices.swap_remove(i as usize);
                indices.pop();
            }
        }

        md[block_index] = (usize::MIN, usize::MIN, u32::MIN, u32::MIN);
        if !last {
            let mut max = (0, 0);
            for (i, (v, _, _, _)) in md.iter().enumerate() {
                if *v > max.1 {
                    max = (i, *v);
                }
            }
            md[max.0] = (vertex_start, vertex_end, index_start, index_end);
        }
    }
}

fn add_xsprite_voxel(
    mesh: &mut Mesh,
    md: &mut XSpriteVIVI,
    block_pos: BlockPos,
    voxel_mesh: &Mesh,
    voxel_dims: Vec3,
    dims: Dimensions,
) {
    let block_index = pos_to_index(block_pos, dims).unwrap();
    let ver_count = mesh.count_vertices();
    let position_offset = voxel_dims * block_pos.as_vec3();
    for (id, vav) in mesh.attributes_mut() {
        if id == Mesh::ATTRIBUTE_POSITION.id {
            let vav2 = voxel_mesh
                .attribute(Mesh::ATTRIBUTE_POSITION.id)
                .unwrap()
                .offset_all(position_offset.into());
            vav.extend(&vav2);
        } else {
            let vav2 = voxel_mesh.attribute(id).unwrap();
            vav.extend(vav2);
        }
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
    md[block_index] = (ver_count, ver_count2, ind_count as u32, ind_count2 as u32);
}

pub fn generate_xsprite_mesh(
    voxel_dims: [f32; 3],
    texture_atlas_dims: [u32; 2],
    texture: [u32; 2],
    voxel_center: [f32; 3],
    padding: f32,
    default_color_intensity: Option<f32>,
    alpha: f32,
    scale: f32,
) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    debug_assert!(
        0.0 <= scale && scale <= 1.0,
        "scale parameter in generate_xsprite_mesh needs to be in 0.0 - 1.0 range"
    );

    let scale = (scale.min(1.0)).max(0.0);

    let z = voxel_center[2] + voxel_dims[2] / 2.0 * scale;
    let nz = voxel_center[2] - voxel_dims[2] / 2.0 * scale;
    let x = voxel_center[0] + voxel_dims[0] / 2.0 * scale;
    let nx = voxel_center[0] - voxel_dims[0] / 2.0 * scale;
    let y = voxel_center[1] + voxel_dims[1] / 2.0;
    let ny = voxel_center[1] - voxel_dims[1] / 2.0;

    let u: f32 = 1.0 / (texture_atlas_dims[0] as f32);
    let v: f32 = 1.0 / (texture_atlas_dims[1] as f32);

    let padding_u = padding / (texture_atlas_dims[0] as f32);
    let padding_v = padding / (texture_atlas_dims[1] as f32);

    let uvs_tl: [f32; 2] = [
        texture[0] as f32 * u + padding_u,
        texture[1] as f32 * v + padding_v,
    ];
    let uvs_br: [f32; 2] = [
        (texture[0] + 1) as f32 * u - padding_u,
        (texture[1] + 1) as f32 * v - padding_v,
    ];

    // Two quads, in the shape of an X, like in Minecraft. This is used for flowers or grass eg.
    #[rustfmt::skip]
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, 
        vec![
            // First sprite
            [nx, y, z],
            [x, y, nz],
            [x, ny, nz],
            [nx, ny, z],
            // Second sprite
            [nx, y, nz],
            [x, y, z],
            [x, ny, z],
            [nx, ny, nz],
        ]
    );

    #[rustfmt::skip]
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, 
        vec![
            uvs_tl, [uvs_br[0], uvs_tl[1]] ,uvs_br, [uvs_tl[0], uvs_br[1]], 
            uvs_tl, [uvs_br[0], uvs_tl[1]] ,uvs_br, [uvs_tl[0], uvs_br[1]], 
        ]
    );

    if let Some(color) = default_color_intensity {
        #[rustfmt::skip]
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![
                [color, color, color, alpha],
                [color, color, color, alpha],
                [color, color, color, alpha],
                [color, color, color, alpha],
                [color, color, color, alpha],
                [color, color, color, alpha],
                [color, color, color, alpha],
                [color, color, color, alpha],
            ]
        );
    }

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            [0.8, 0.0, 0.8],
            [0.8, 0.0, 0.8],
            [0.8, 0.0, 0.8],
            [0.8, 0.0, 0.8],
            [-0.8, 0.0, 0.8],
            [-0.8, 0.0, 0.8],
            [-0.8, 0.0, 0.8],
            [-0.8, 0.0, 0.8],
        ],
    );

    mesh.set_indices(Some(Indices::U32(vec![0, 1, 3, 2, 3, 1, 4, 5, 7, 6, 7, 5])));

    mesh
}
