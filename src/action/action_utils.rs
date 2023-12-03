use crate::chunk::{ChunkCords, ChunkMap, ToApplySL, CHUNK_DIMS, CHUNK_TOTAL_BLOCKS_USIZE};
use crate::prelude::notical;

use super::*;

// Inserts component `ToApplySL` to adj chunks after block update
pub(super) fn asl2ac(
    commands: &mut Commands,
    block_pos: BlockPos,
    cords: ChunkCords,
    chunk_map: &ChunkMap,
    num_of_events_per_frame: usize,
) {
    for i in 0..6 {
        let face = Face::from(i);
        if is_block_pos_on_edge(block_pos, face, CHUNK_DIMS) {
            let change = to_cords(Some(notical::Direction::from(face)));
            let new_cords = [cords[0] + change[0], cords[1] + change[1]];
            if let Some(neighboring_entity) = chunk_map.pos_to_ent.get(&ChunkCords::from(new_cords))
            {
                if *neighboring_entity == Entity::PLACEHOLDER {
                    continue;
                }
                if num_of_events_per_frame == 1 {
                    commands.entity(*neighboring_entity).insert(ToApplySL(
                        block_pos.wrapping_sub(UVec3::Y * 2),
                        block_pos.wrapping_add(UVec3::Y * 2),
                    ));
                } else {
                    commands.entity(*neighboring_entity).insert(ToApplySL(
                        index_to_pos(0, CHUNK_DIMS).unwrap(),
                        index_to_pos(CHUNK_TOTAL_BLOCKS_USIZE, CHUNK_DIMS).unwrap(),
                    ));
                }
            }
        }
    }
}
