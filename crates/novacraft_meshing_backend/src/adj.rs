use crate::prelude::*;
use bevy::prelude::*;

/// This function will iron out any problems caused by generating two chunks that are adjacent to
/// each other seperatly. For example, it will cull the unneeded vertices between them, and fix
/// the proximity based shadowing.
/// reg: the Voxel Registry
/// main_mesh: the mesh to change
/// main_md: the metadata of the mesh to change
/// connection_side: from the POV of the main mesh, where is the adjacent mesh?
/// adjacent_chunk_grid: the grid of the chunk to introduce
pub fn introduce_adjacent_chunks<T: std::fmt::Debug + Sized + Copy, const N: usize>(
    reg: &impl VoxelRegistry<Voxel = T>,
    main_mesh: &mut Mesh,
    main_md: &mut MeshMD<T>,
    connection_side: Face,
    adjacent_chunk_grid: &ChunkGrid<T, N>,
) {
    assert_eq!(
        adjacent_chunk_grid.len(),
        main_md.vivi.vivi.len(),
        "Cannot introduce chunks with different sizes to each other"
    );
    let dims = main_md.dims;
    for voxel_pos in adjacent_chunk_grid.iter_blocks_on_edge(connection_side) {
        let adj_voxel_pos = neighbor_across_chunk(voxel_pos, connection_side, dims);
        let adj_voxel_pos = if adj_voxel_pos.is_some() {
            adj_voxel_pos.unwrap()
        } else {
            continue;
        };

        let adj_voxel = adjacent_chunk_grid.get_block(adj_voxel_pos).unwrap();
        if reg.is_covering(&adj_voxel, connection_side.opposite()) {
            let mut tmp = [None; 6];
            tmp[connection_side as usize] = Some(adj_voxel);
            main_md.log(VoxelChange::CullFaces, voxel_pos, adj_voxel, tmp)
        }
    }
    update_mesh(main_mesh, main_md, reg);
}
