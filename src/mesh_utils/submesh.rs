use crate::blocks::*;
use crate::prelude::*;

#[derive(Copy, Clone)]
pub struct SubMeshGrid<'a> {
    pub sub_grid_dims: Dimensions,
    pub full_grid_dims: Dimensions,
    pub south_west_down_corner: [usize; 3],
    pub meshing_algorithm: MeshingAlgorithm,
    pub smooth_lighting_params: Option<SmoothLightingParameters>,
    pub outer_layers_to_cull: &'a [Face],
}
// chunks: 9 grids, the first one (index = 0) is the main one, the rest are corresponding to their
//   direction from the main one (Direction as usize).
pub fn get_submesh_of_grid(
    reg: &impl VoxelRegistry<Voxel = Block>,
    sub_mesh_grid: SubMeshGrid,
    chunks: Vec<&[Block]>,
) -> Option<Mesh> {
    let SubMeshGrid {
        sub_grid_dims,
        full_grid_dims,
        south_west_down_corner,
        meshing_algorithm,
        smooth_lighting_params,
        outer_layers_to_cull,
    } = sub_mesh_grid;
    let dims = sub_grid_dims;
    let sub_grid_len = dims.0 * dims.1 * dims.2;
    let mut sub_grid = vec![Block::AIR; sub_grid_len];
    let swdc = one_d_cords(south_west_down_corner, full_grid_dims);
    for y in 0..dims.1 - 1 {
        for z in 0..dims.2 - 1 {
            for x in 0..dims.0 - 1 {
                let sub_grid_cords = one_d_cords([x, y, z], dims);
                sub_grid[sub_grid_cords] =
                    match get_block_n_away(full_grid_dims, swdc, x as i32, y as i32, z as i32) {
                        None => Block::AIR,
                        Some((direction, index)) => match direction {
                            None => chunks[0][index],
                            Some(dir) => chunks[dir as usize + 1][index],
                        },
                    };
            }
        }
    }
    return if let Some((mesh, _)) = mesh_grid(
        dims,
        outer_layers_to_cull,
        sub_grid.as_slice(),
        reg,
        meshing_algorithm,
        smooth_lighting_params,
    ) {
        Some(mesh)
    } else {
        None
    };
}
