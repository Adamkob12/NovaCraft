//! This module contains the main functions themself, and some added utilities and defs.
use crate::pbs::*;
use crate::prelude::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, MeshVertexAttribute, VertexAttributeValues};
use bevy::render::render_resource::PrimitiveTopology;

/// All the variants for the Meshing algorithm.
#[derive(Debug, Clone, Copy)]
pub enum MeshingAlgorithm {
    Naive,
    Culling,
}

/// Arguments:
/// - [`grid`](&[T]): one dimentional slice of voxels, to turn into a single mesh, the function
///     assumes the real grid is 3 dimentional, and that the width, height and length match the
///     dimensions given with the dims argument.
/// - [`reg`](VoxelRegistry): this is a data structure that will return the desired mesh attribute
///     we need, but(!) the size of each of the voxels MUST be the same across the entire grid.
///     if this condition is not met, the grid will not be properly meshified.
///     An example to create a [`VoxelRegistry`] is in the examples folder.
/// - ['ma'](MeshingAlgorithm): The meshing algorithm to use - currently supports Culling and
///     Naive. (Culling is always better than Naive)
/// - ['sl']: Enable Smooth Lighting (Some ..) or not (None). Smooth Lighting is a technique often used in
///     voxel based games that resembles Ambient Occlusion, but it is static- which means the
///     shadows are computed only once, when the mesh is generated (or updated).
///
/// Return:
/// - The first mesh is the mesh of the full, normal cube voxels. (for example, the stone blocks)
/// - MeshMD<T> is the mesh metadata that the user needs to keep if they want to update the mesh.
/// - None: Couldn't generate the mesh
pub fn meshify_cubic_voxels<T: Copy, const N: usize>(
    outer_layer: &[Face],
    grid: &ChunkGrid<T, N>,
    reg: &impl VoxelRegistry<Voxel = T>,
    meshing_algorithm: MeshingAlgorithm,
    smooth_lighting_params: Option<SmoothLightingParameters>,
) -> Option<(Mesh, MeshMD<T>)> {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let total_voxels = grid.len();
    let mut vivi = VIVI::new(total_voxels);
    let mut outer_layers_to_call = [true, true, true, true, true, true];
    for f in outer_layer {
        outer_layers_to_call[*f as usize] = false;
    }

    let mut indices: Vec<u32> = vec![];
    let mut vertices: Vec<(MeshVertexAttribute, VertexAttributeValues)> = vec![];
    for att in reg.all_attributes().iter() {
        vertices.push((att.clone(), VertexAttributeValues::new(att.format.clone())));
    }
    let voxel_dims = reg.get_voxel_dimensions();
    let center = reg.get_center();

    for (voxel_pos, voxel) in grid.enumerate_blocks() {
        let position_offset = Vec3::from(voxel_dims) * voxel_pos.as_vec3();

        let sides_to_cull = match meshing_algorithm {
            MeshingAlgorithm::Culling => grid.enumerate_neighbors(voxel_pos).map(|(f, n)| {
                n.map_or_else(
                    || outer_layers_to_call[f as usize],
                    |t| !reg.is_covering(&t, f.opposite()),
                )
            }),
            MeshingAlgorithm::Naive => [true; 6],
        };

        if sides_to_cull == [false; 6] {
            continue;
        }

        if let VoxelMesh::NormalCube(voxel_mesh) = reg.get_mesh(&voxel) {
            add_vertices_normal_cube(
                sides_to_cull,
                &mut indices,
                &mut vertices,
                voxel_mesh,
                &mut vivi,
                voxel_pos,
                center,
                position_offset.into(),
                grid.dims,
            );
        }
    }

    for (att, vals) in vertices {
        mesh.insert_attribute(att, vals);
    }
    mesh.set_indices(Some(Indices::U32(indices)));

    let d_mesh = MeshMD {
        dims: grid.dims,
        smooth_lighting_params,
        vivi,
        changed_voxels: vec![],
    };

    if let Some(t) = smooth_lighting_params {
        if t.apply_at_gen {
            apply_smooth_lighting(reg, &mut mesh, &d_mesh, grid.dims, 0, total_voxels, grid);
        }
    }
    Some((mesh, d_mesh))
}

/// Important helper function to add the vertices and indices of each voxel into the running count of vertices
/// and indices, preserving their attributes, and (important!) assigning a custom offset to the
/// position attributes, we are assuming this is only needed for the position attributes (because
/// it usually is).
fn add_vertices_normal_cube(
    neig: Neighbors,
    indices_main: &mut Vec<u32>,
    vertices: &mut Vec<(MeshVertexAttribute, VertexAttributeValues)>,
    voxel: &Mesh,
    vivi: &mut VIVI,
    voxel_pos: BlockPos,
    center: [f32; 3],
    position_offset: (f32, f32, f32),
    dims: Dimensions,
) {
    let vertices_count = vertices[0].1.len();
    let pos_attribute = voxel
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("couldn't get voxel mesh data");
    let VertexAttributeValues::Float32x3(positions) = pos_attribute else {
        panic!("Unexpected vertex format for position attribute, expected Float32x3.");
    };
    let Indices::U32(indices) = voxel.indices().expect("couldn't get indices data") else {
        panic!("Expected U32 indices format");
    };
    let triangles = indices
        .chunks(3)
        .map(|chunk| (chunk[0], chunk[1], chunk[2]));

    // define the indices and vertices we want to save of the voxel mesh
    let mut indices_to_save: Vec<u32> = vec![];
    // helper data structure
    let mut vertices_to_save: Vec<(bool, u32, Face)> = vec![(false, 0, Face::Top); positions.len()];
    // sorted vertices by the quad they are in
    let mut sorted_vertices: Vec<Option<Vec<u32>>> = vec![None; 6];
    // the final array of the vertices, it will be sorted, each 4 vertices will be a
    // part of one quad, we sort them this way to efficiently update the vivi.
    let mut final_vertices: Vec<u32> = vec![];

    use Face::*;
    // iterate over all the triangles in the mesh
    for (a, b, c) in triangles {
        let v1 = positions[a as usize];
        let v2 = positions[b as usize];
        let v3 = positions[c as usize];
        let mut save = (false, Top);

        // see which side of the voxel the triangle belongs to
        for i in 0..3 {
            if v1[i] == v2[i] && v2[i] == v3[i] && v1[i] == v3[i] {
                match (i, center[i] > v1[i]) {
                    (0, true) if neig[3] => save = (true, Left),
                    (0, false) if neig[2] => save = (true, Right),
                    (1, true) if neig[1] => save = (true, Bottom),
                    (1, false) if neig[0] => save = (true, Top),
                    (2, true) if neig[5] => save = (true, Forward),
                    (2, false) if neig[4] => save = (true, Back),
                    _ => save = (false, Top),
                }
                break;
            }
        }

        // save the vertices
        if save.0 {
            let quad: usize = save.1.into();
            indices_to_save.push(a);
            indices_to_save.push(b);
            indices_to_save.push(c);
            match sorted_vertices[quad] {
                None => {
                    sorted_vertices[quad] = Some(vec![a, b, c]);
                    vertices_to_save[a as usize].0 = true;
                    vertices_to_save[b as usize].0 = true;
                    vertices_to_save[c as usize].0 = true;
                    vertices_to_save[a as usize].1 = 0;
                    vertices_to_save[b as usize].1 = 1;
                    vertices_to_save[c as usize].1 = 2;
                    vertices_to_save[a as usize].2 = save.1;
                    vertices_to_save[b as usize].2 = save.1;
                    vertices_to_save[c as usize].2 = save.1;
                }
                Some(ref mut v) => {
                    for &i in [a, b, c].iter() {
                        if !vertices_to_save[i as usize].0 {
                            v.push(i);
                            vertices_to_save[i as usize].2 = save.1;
                            vertices_to_save[i as usize].1 = v.len() as u32 - 1;
                            vertices_to_save[i as usize].0 = true;
                        }
                    }
                }
            }
        }
    }

    // The code from now on is a little messy, but it is very simple in actuality. It is mostly
    // just offseting the vertices and indices and formatting them into the right data-structres.

    // offset the vertices, since we won't be using all the vertices of the the mesh,
    // we need to find out which of them we will be using first, and then filter out
    // the ones we dont need.
    let mut offset: u32 = 0;
    for q in sorted_vertices.iter() {
        match q {
            None => offset += 4,
            Some(ref v) => {
                let mut only_first = true;
                for &i in v.iter() {
                    let face = vertices_to_save[i as usize].2;
                    vertices_to_save[i as usize].1 += face as u32 * 4 - offset;
                    final_vertices.push(i);
                    // update the vivi
                    if only_first {
                        vivi.insert(
                            face,
                            pos_to_index(voxel_pos, dims).unwrap(),
                            i + vertices_count as u32 - offset,
                        );
                        only_first = false;
                    }
                }
            }
        }
    }

    // offset the indices, we need to consider the fact that the indices wil be part of a big mesh,
    // with a lot of vertices, so we must the vertices to a running count and offset them accordingly.
    for i in indices_to_save.iter_mut() {
        *i = vertices_to_save[*i as usize].1 + vertices_count as u32;
    }

    for (id, vals) in vertices.iter_mut() {
        let mut att = voxel
            .attribute(id.id)
            .expect(format!("Couldn't retrieve voxel mesh attribute {:?}.", id).as_str())
            .get_needed(&final_vertices);
        if id.id == Mesh::ATTRIBUTE_POSITION.id {
            att = att.offset_all(position_offset);
        }
        vals.extend(&att);
    }
    indices_main.extend(indices_to_save);
}
