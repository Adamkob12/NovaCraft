use super::{chunkmd::CMMD, *};
use crate::blocks::blockreg::BlockRegistry;

// "Introduce" means cull the sides between the chunks (the intersection). And apply Smooth
// Lighting if needed.
pub(super) fn introduce_neighboring_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_query: Query<(&Handle<Mesh>, &CMMD)>,
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
        let mut to_remove = [false; 8];
        for (_adj_cords, direction) in to_introduce.0.iter() {
            'A: for child in children {
                if let Ok((mesh_handle, metadata)) = mesh_query.get(*child) {
                    let mesh_ref_mut = meshes.get_mut(mesh_handle).unwrap();
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
                        match direction {
                            North | South | West | East => {
                                introduce_adjacent_chunks(
                                    Arc::clone(&breg).as_ref(),
                                    mesh_ref_mut,
                                    &mut metadata
                                        .0
                                        .write()
                                        .expect("a")
                                        .extract_meshmd_mut()
                                        .unwrap(),
                                    (*direction).into(),
                                    adj_grid.read().expect("b").as_ref(),
                                );
                            }
                            _ => {}
                        }
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
            let mut tmp = commands.entity(entity);
            tmp.remove::<ToIntroduce>();
        }
    }
}
