// REFACTORED

use super::{chunk_queue::ChunkQueue, *};
use crate::{action::RigidLayer, utils::chunk_distance};
use bevy_xpbd_3d::prelude::{CollisionLayers, ComputedCollider, RigidBody, TriMeshFlags};

pub(super) fn despawn_all_chunks(
    mut commands: Commands,
    mut chunk_map: ResMut<ChunkMap>,
    chunk_queue: ResMut<ChunkQueue>,
    chunks_query: Query<Entity, With<ParentChunk>>,
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

/// A system that "connects" chunks. "connecting" chunk A and chunk B means
/// clone the grid of chunk A ([`Arc`]) and put it in the corresponding place
/// in the [`AdjChunkGrids`] of the chunk.
pub(super) fn connect_chunks(
    chunk_map: Res<ChunkMap>,
    chunk_grid_query: Query<&Grid>,
    mut chunk_data_query: Query<(Entity, &mut AdjChunkGrids, &Cords), With<ToConnect>>,
    mut commands: Commands,
) {
    for (entity, mut adj_chunk_grids, cords) in chunk_data_query.iter_mut() {
        // if rand::thread_rng().gen::<f32>() > 0.2 {
        //     continue;
        // }
        for direction in DIRECTIONS {
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

/// Chunks that are within a chunk's distance away from the player are [`CloseChunk`]
/// This system updates the chunks with [`CloseChunk`] as the player moves.
pub(super) fn update_close_chunks(
    mut commands: Commands,
    chunks: Query<(Entity, &Cords, Has<CloseChunk>), With<ParentChunk>>,
    current_chunk: Res<CurrentChunk>,
) {
    let current_chunk = current_chunk.0;
    for (entity, Cords(cords), close) in chunks.iter() {
        if !close && chunk_distance(current_chunk, *cords) < 2 {
            commands.entity(entity).insert(CloseChunk);
        }
        if close && chunk_distance(current_chunk, *cords) > 1 {
            commands.entity(entity).remove::<CloseChunk>();
        }
    }
}

/// This system inserts the collider for newly added [`CloseChunk`]
pub(super) fn insert_collider_for_close_chunks(
    mut commands: Commands,
    new_close_chunks: Query<&Children, Added<CloseChunk>>,
    subchunks_query: Query<(Entity, &ChunkRigidLayers), With<Subchunk>>,
) {
    for subchunks in new_close_chunks.iter() {
        for &subchunk_entity in subchunks {
            if let Ok((entity, rigid_layers)) = subchunks_query.get(subchunk_entity) {
                commands.entity(entity).insert((
                    RigidBody::Static,
                    AsyncCollider(ComputedCollider::TriMeshWithFlags(
                        TriMeshFlags::MERGE_DUPLICATE_VERTICES,
                    )),
                    CollisionLayers::all_masks::<RigidLayer>().add_groups(rigid_layers.0.clone()),
                ));
            }
        }
    }
}

// pub(super) fn remove_collider_for_far_away_chunks(
//     mut commands: Commands,
//     mut far_away_chunks: RemovedComponents<CloseChunk>,
//     parent_query: Query<&MainChild>,
// ) {
//     for far_away_chunk in far_away_chunks.read() {
//         if let Ok(MainChild(child)) = parent_query.get(far_away_chunk) {
//             commands
//                 .entity(*child)
//                 .remove::<RigidBody>()
//                 .remove::<Collider>();
//         }
//     }
// }
