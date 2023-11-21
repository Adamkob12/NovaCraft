use crate::mesh_utils::{
    ChunkCords, ChunkMap, ToApplySL, CHUNK_DIMS, CHUNK_TOTAL_BLOCKS, LENGTH, WIDTH,
};
use crate::prelude::notical;

use super::*;

// Inserts component `ToApplySL` to adj chunks after block update
pub(super) fn asl2ac(
    commands: &mut Commands,
    index: usize,
    cords: ChunkCords,
    chunk_map: &ChunkMap,
    num_of_events_per_frame: usize,
) {
    for i in 0..6 {
        let face = Face::from(i);
        if is_block_on_edge(CHUNK_DIMS, index, face) {
            let change = to_cords(Some(notical::Direction::from(face)));
            let new_cords = [cords[0] + change[0], cords[1] + change[1]];
            if let Some(neighboring_entity) = chunk_map.pos_to_ent.get(&new_cords) {
                if *neighboring_entity == Entity::PLACEHOLDER {
                    continue;
                }
                if num_of_events_per_frame == 1 {
                    commands.entity(*neighboring_entity).insert(ToApplySL(
                        index.checked_sub(WIDTH * LENGTH * 2).unwrap_or(0),
                        index + (WIDTH * LENGTH * 2),
                    ));
                } else {
                    commands
                        .entity(*neighboring_entity)
                        .insert(ToApplySL(0, CHUNK_TOTAL_BLOCKS));
                }
            }
        }
    }
}
