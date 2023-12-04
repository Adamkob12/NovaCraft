// REFACTORED

use crate::chunk::{ChunkCords, ChunkMap, ToApplySL, CHUNK_DIMS, CHUNK_TOTAL_BLOCKS_USIZE};
use crate::prelude::notical;

use super::*;

// Helper function (not system) that inserts component `ToApplySL` to adj chunks
pub(super) fn apply_smooth_lighting_util(
    commands: &mut Commands,
    block_pos: BlockPos,
    cords: ChunkCords,
    chunk_map: &ChunkMap,
    num_of_events_per_frame: usize,
) {
    for face in FACES {
        if is_block_pos_on_edge(block_pos, face, CHUNK_DIMS) {
            let change = to_cords(Some(notical::Direction::from(face)));
            let new_cords = [cords[0] + change[0], cords[1] + change[1]];
            if let Some(adj_chunk_entity) = chunk_map.pos_to_ent.get(&ChunkCords::from(new_cords)) {
                if *adj_chunk_entity == Entity::PLACEHOLDER {
                    continue;
                }
                if num_of_events_per_frame == 1 {
                    commands.entity(*adj_chunk_entity).insert(ToApplySL(
                        block_pos.wrapping_sub(UVec3::Y * 2),
                        block_pos.wrapping_add(UVec3::Y * 2),
                    ));
                } else {
                    commands.entity(*adj_chunk_entity).insert(ToApplySL(
                        index_to_pos(0, CHUNK_DIMS).unwrap(),
                        index_to_pos(CHUNK_TOTAL_BLOCKS_USIZE, CHUNK_DIMS).unwrap(),
                    ));
                }
            }
        }
    }
}

/// Helper function (not system) that sends a defined world update to a position,
/// and 6 undefined world updates in the surrounding directions.
pub fn send_world_updates_surrounding_blocks(
    block_pos: BlockPos,
    chunk_cords: ChunkCords,
    world_block_update_sender: &mut EventWriter<WorldBlockUpdate>,
    block_update: BlockUpdate,
) {
    let global_pos = BlockGlobalPos::new(block_pos, chunk_cords);
    for (_face, neighbor_global_pos) in global_enumerate_neighboring_blocks(global_pos, CHUNK_DIMS)
    {
        world_block_update_sender.send(WorldBlockUpdate::from_global_pos(neighbor_global_pos));
    }
    world_block_update_sender.send(WorldBlockUpdate {
        block_pos,
        chunk_cords,
        block_update: Some(block_update),
    });
}
