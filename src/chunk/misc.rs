use super::{chunk_queue::ChunkQueue, *};

pub(super) fn reload_all(
    mut commands: Commands,
    mut chunk_map: ResMut<ChunkMap>,
    chunk_queue: ResMut<ChunkQueue>,
    chunks_query: Query<Entity, With<Chunk>>,
    tasks_query: Query<Entity, With<ComputeChunk>>,
) {
    chunk_queue.into_inner().clear();
    chunk_map.pos_to_ent.clear();
    for chunk_entity in chunks_query.iter() {
        commands.entity(chunk_entity).despawn_recursive();
    }
    for task_entity in tasks_query.iter() {
        commands.entity(task_entity).despawn_recursive();
    }
}

pub(super) fn connect_chunks(
    chunk_map: Res<ChunkMap>,
    chunk_grid_query: Query<&Grid>,
    mut chunk_data_query: Query<(Entity, &mut AdjChunkGrids, &Cords), With<ToConnect>>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    for (entity, mut adj_chunk_grids, cords) in chunk_data_query.iter_mut() {
        let p: f32 = rng.gen();
        if p > 0.2 {
            continue;
        }
        for direction in 1..9 {
            let direction = Direction::from(direction);
            let adj_chunk_cords = get_neighboring_chunk_cords(cords.0, direction);
            if let Some(adj_entity) = chunk_map.pos_to_ent.get(&adj_chunk_cords) {
                if *adj_entity == Entity::PLACEHOLDER {
                    continue;
                }
                if let Ok(Grid(adj_grid)) = chunk_grid_query.get(*adj_entity) {
                    match direction {
                        North => adj_chunk_grids.north = Some(Arc::clone(adj_grid)),
                        South => adj_chunk_grids.south = Some(Arc::clone(adj_grid)),
                        East => adj_chunk_grids.east = Some(Arc::clone(adj_grid)),
                        West => adj_chunk_grids.west = Some(Arc::clone(adj_grid)),
                        NoEast => adj_chunk_grids.no_east = Some(Arc::clone(adj_grid)),
                        NoWest => adj_chunk_grids.no_west = Some(Arc::clone(adj_grid)),
                        SoEast => adj_chunk_grids.so_east = Some(Arc::clone(adj_grid)),
                        SoWest => adj_chunk_grids.so_west = Some(Arc::clone(adj_grid)),
                    }
                }
                if adj_chunk_grids.north.is_some()
                    && adj_chunk_grids.south.is_some()
                    && adj_chunk_grids.west.is_some()
                    && adj_chunk_grids.east.is_some()
                    && adj_chunk_grids.no_east.is_some()
                    && adj_chunk_grids.no_west.is_some()
                    && adj_chunk_grids.so_east.is_some()
                    && adj_chunk_grids.so_west.is_some()
                {
                    commands.entity(entity).remove::<ToConnect>();
                }
            }
        }
    }
}
