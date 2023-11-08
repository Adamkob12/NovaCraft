use super::{chunk_queue::ChunkQueue, *};
use crate::{blocks::blockreg::BlockRegistry, chunk::XSpriteMesh, utils::chunk_distance};

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
