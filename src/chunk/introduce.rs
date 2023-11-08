use super::*;
use crate::blocks::blockreg::BlockRegistry;
use rand::prelude::*;

// "Introduce" means cull the sides between the chunks, which aren't visible, and apply pbs.
pub(super) fn introduce_neighboring_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_query: Query<(&Handle<Mesh>, &MainCulledMesh)>,
    mut to_introduce_query: Query<(Entity, &Children, &mut ToIntroduce, &AdjChunkGrids)>,
    breg: Res<BlockRegistry>,
) {
    let mut rng = rand::thread_rng();
    let breg = Arc::new(breg.into_inner().to_owned());
    for (entity, children, mut to_introduce, adj_chunk_grids) in to_introduce_query.iter_mut() {
        let p: f32 = rng.gen();
        if p > 0.1 {
            continue;
        }
        let mut to_remove = [false; 6];
        for (_adj_cords, direction) in to_introduce.0.iter() {
            'A: for child in children {
                if let Ok((mesh_handle, MainCulledMesh(metadata))) = mesh_query.get(*child) {
                    let mesh_ref_mut = meshes.get_mut(mesh_handle).unwrap();
                    if let Some(adj_grid) = match direction {
                        Back => &adj_chunk_grids.north,
                        Forward => &adj_chunk_grids.south,
                        Right => &adj_chunk_grids.east,
                        Left => &adj_chunk_grids.west,
                        _ => unreachable!(),
                    } {
                        introduce_adjacent_chunks(
                            Arc::clone(&breg).as_ref(),
                            mesh_ref_mut,
                            &mut metadata.write().unwrap(),
                            *direction,
                            adj_grid.as_ref().read().unwrap().as_ref(),
                        );
                        break 'A;
                    } else {
                        to_remove[*direction as usize] = true;
                    }
                }
            }
        }
        to_introduce.0 = to_introduce
            .0
            .iter()
            .copied()
            .filter(|(_, y)| to_remove[*y as usize])
            .collect();
        if to_introduce.0.is_empty() {
            commands.entity(entity).remove::<ToIntroduce>();
        }
    }
}
