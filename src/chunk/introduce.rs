use std::borrow::BorrowMut;

use crate::blocks::blockreg::BlockRegistry;

use super::*;

// "Introduce" means cull the sides between the chunks, which aren't visible, and apply pbs.
pub fn introduce_neighboring_chunks_system(
    chunk_map: Res<ChunkMap>,
    mut commands: Commands,
    to_intoduce_query: Query<(Entity, &Cords, &ToIntroduce)>,
    chunk_query: Query<(&Children, &Grid, &MetaData)>,
    mut meshes: ResMut<Assets<Mesh>>,
    main_mesh_query: Query<&Handle<Mesh>, With<MainCulledMesh>>,
    breg: Res<BlockRegistry>,
) {
    let breg = Arc::new(breg.into_inner().to_owned());
    for (entity, _cords, to_intoduce) in to_intoduce_query.iter() {
        if let Ok((children, grid, metadata)) = chunk_query.get(entity) {
            for child in children {
                if let Ok(mesh_handle) = main_mesh_query.get(*child) {
                    for (adj_cords, adj_face) in to_intoduce.0.iter() {
                        let adj_entity = *chunk_map
                            .pos_to_ent
                            .get(adj_cords)
                            .unwrap_or(&Entity::PLACEHOLDER);
                        if adj_entity == Entity::PLACEHOLDER {
                            continue;
                        } else {
                            if let Ok((adj_children, _, adj_metadata)) = chunk_query.get(adj_entity)
                            {
                                for adj_child in adj_children {
                                    if let Ok(adj_mesh_handle) = main_mesh_query.get(*adj_child) {
                                        let adj_mesh = meshes.get_mut(adj_mesh_handle).unwrap();
                                        introduce_adjacent_chunks(
                                            Arc::clone(&breg).as_ref(),
                                            adj_mesh,
                                            &mut adj_metadata.0.write().unwrap(),
                                            adj_face.opposite(),
                                            &grid.0,
                                        );
                                    }
                                }
                            }
                        }
                        let mesh = meshes.get_mut(mesh_handle).unwrap();
                        let (_, adj_grid, _) = chunk_query.get(adj_entity)
                            .expect("Entity that wasn't in World was found in internal data structure ChunkMap, but shouldn't be.");
                        introduce_adjacent_chunks(
                            Arc::clone(&breg).as_ref(),
                            mesh,
                            &mut metadata.0.write().unwrap(),
                            *adj_face,
                            &adj_grid.0,
                        );
                    }
                }
            }
        }
        commands.entity(entity).remove::<ToIntroduce>();
    }
}
