// REFACTORED

use super::{chunkmd::SubChunkMD, *};
use crate::blocks::MeshRegistry;

/// "Introduce" means cull the sides between the chunks (the intersection). And apply Smooth
/// Lighting if needed.
pub(super) fn introduce_neighboring_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_query: Query<(&Handle<Mesh>, &SubChunkMD), With<CubeSubChunk>>,
    mut to_introduce_query: Query<(Entity, &Children, &mut ToIntroduce, &AdjChunkGrids)>,
    mreg: Res<MeshRegistry>,
) {
    let mreg = Arc::new(mreg.into_inner().to_owned());
    for (chunk_entity, subchunks, mut to_introduce, adj_chunk_grids) in
        to_introduce_query.iter_mut()
    {
        // optimization method. we dont want to overload the system with a lot of chunks to
        // compute at once, so we only compute 1/12 of them each frame. This is ok because introducing
        // is not a critical operation that needs to be executed right away.
        if rand::thread_rng().gen::<f32>() > 0.08 {
            continue;
        }
        let mut introduced = [false; 8];
        for subchunk in subchunks {
            if let Ok((mesh_handle, metadata)) = mesh_query.get(*subchunk) {
                let mesh_ref_mut = meshes.get_mut(mesh_handle).unwrap();
                // iterate over all of the directions (that represebt adjecant chunks we need to introduce)
                for (_, direction) in to_introduce.0.iter() {
                    if let Some(adj_grid) = match direction {
                        North => &adj_chunk_grids.north,
                        South => &adj_chunk_grids.south,
                        East => &adj_chunk_grids.east,
                        West => &adj_chunk_grids.west,
                        NoEast => &adj_chunk_grids.no_east,
                        NoWest => &adj_chunk_grids.no_west,
                        SoEast => &adj_chunk_grids.so_east,
                        SoWest => &adj_chunk_grids.so_west,
                    } {
                        introduced[(*direction) as usize] = true;
                        match direction {
                            North | South | West | East => {
                                introduce_adjacent_chunks(
                                    Arc::clone(&mreg).as_ref(),
                                    mesh_ref_mut,
                                    &mut metadata
                                        .0
                                        .write()
                                        .expect("a")
                                        .extract_meshmd_mut()
                                        .unwrap(),
                                    (*direction).into(),
                                    &adj_grid.read().expect("b"),
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        // update the adjecant chunks that we introduced.
        to_introduce.0 = to_introduce
            .0
            .iter()
            .copied()
            .filter(|(_, y)| !introduced[*y as usize])
            .collect();
        // If there are no more adjecant chunks to intreoduce, remove the component
        if to_introduce.0.is_empty() {
            commands.entity(chunk_entity).remove::<ToIntroduce>();
        }
    }
}
