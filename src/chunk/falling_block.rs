use bevy_xpbd_3d::prelude::ShapeHits;

use crate::action::{properties::FallingBlock, PlaceBlockGlobalEvent};

use super::*;

pub fn follow_falling_block(
    mut commands: Commands,
    falling_blocks: Query<(Entity, &ShapeHits, &Block, &Transform), With<FallingBlock>>,
    mut global_block_place_event_sender: EventWriter<PlaceBlockGlobalEvent>,
) {
    for (entity, hits, block, transform) in falling_blocks.iter() {
        if !hits.is_empty() {
            commands.entity(entity).despawn();
            let (chunk_pos, block_index, flag) =
                position_to_chunk_position(transform.translation + Vec3::Y * 0.1, CHUNK_DIMS);
            if flag {
                global_block_place_event_sender.send(PlaceBlockGlobalEvent {
                    block: *block,
                    chunk_pos,
                    block_index: one_d_cords(block_index, CHUNK_DIMS),
                })
            }
        }
    }
}
