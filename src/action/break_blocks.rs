// REFACTORED

use super::*;
use crate::chunk::{
    chunkmd::{MetaData, SubChunkMD},
    ChunkMap, Cords, CubeChild, CubeSubChunk, Grid, ParentChunk, ToUpdate, CHUNK_DIMS,
};

/// The final event in the block-breaking pipeline. The modular design of the pipeline
/// allows for this event to be called from all over the code. This event will be processed
/// by the `global_block_breaker` system.
#[derive(Event)]
pub struct BreakBlockGlobalEvent {
    chunk_cords: Option<ChunkCords>,
    chunk_entity: Option<Entity>,
    block_pos: BlockPos,
}

#[allow(dead_code)]
impl BreakBlockGlobalEvent {
    /// Create an event from a point in space, the block that contains that point will be broken.
    pub fn from_point(point: Vec3) -> Option<Self> {
        let BlockGlobalPos {
            pos,
            chunk_cords,
            valid,
        } = point_to_global_block_pos(point, CHUNK_DIMS);
        if !valid {
            return None;
        }
        Some(Self {
            chunk_cords: Some(chunk_cords),
            chunk_entity: None,
            block_pos: pos,
        })
    }

    /// Create an event from the global position of the block
    pub fn from_global_pos(global_pos: BlockGlobalPos) -> Self {
        Self {
            block_pos: global_pos.pos,
            chunk_cords: Some(global_pos.chunk_cords),
            chunk_entity: None,
        }
    }

    /// Create an event from the position of the block and the
    /// entity of the chunk the block is.
    pub fn from_entity_and_pos(block_pos: BlockPos, chunk_entity: Entity) -> Self {
        Self {
            block_pos,
            chunk_entity: Some(chunk_entity),
            chunk_cords: None,
        }
    }
}

/// This system executes once every frame. It is the final stage of the block breaking pipeline, and after its
/// execution, block-breaking is unreversable. It processes all of the pending `BreakBlockGlobalEvent`(s)
/// and marks the chunks that need to be updated. The meshes those chunks will be updated thereafter.
pub fn global_block_breaker(
    mut global_block_break_events: EventReader<BreakBlockGlobalEvent>,
    mut world_block_update_sender: EventWriter<WorldBlockUpdate>,
    mut commands: Commands,
    chunk_map: Res<ChunkMap>,
    parent_chunks: Query<(&Grid, &Cords, &Children, &CubeChild), With<ParentChunk>>,
    chunk_metadata: Query<(&SubChunkMD, &Parent, Has<CubeSubChunk>)>,
) {
    let len = global_block_break_events.len();
    for global_block_break in global_block_break_events.read() {
        let BreakBlockGlobalEvent {
            chunk_entity,
            block_pos,
            chunk_cords,
        } = *global_block_break;
        // Get the parent chunk using the entity or the cords.
        if let Some(parent_chunk) = chunk_entity.map_or(
            chunk_cords.map(|cords| chunk_map.pos_to_ent.get(&cords).copied().unwrap()),
            |e| {
                chunk_metadata
                    .get(e)
                    .ok()
                    .map(|(_, parent, _)| parent.get())
            },
        ) {
            let (Grid(chunk_grid), &Cords(chunk_cords), subchunks, _) =
                parent_chunks.get(parent_chunk).unwrap();
            let _ = chunk_grid.write().unwrap().set_block(Block::AIR, block_pos);

            for subchunk in subchunks {
                if let Ok((subchunk_md, _, cube_chunk)) = chunk_metadata.get(*subchunk) {
                    // Update the metadata to reflect the changes we want to make to the mesh.
                    // Note: we update *all* the sub-chunks to break the block at that position,
                    // even though only one has actually changed. It's simpler and it won't cost
                    // anything, as if a change has been reported without an actual change, nothing
                    // will happen.
                    subchunk_md.0.write().unwrap().log_break(
                        block_pos,
                        chunk_grid.read().unwrap().get_neighbors(block_pos),
                    );

                    // Mark the sub-chunk to update, and update the smooth lighting.
                    commands.entity(*subchunk).insert(ToUpdate);
                    apply_smooth_lighting_util(
                        &mut commands,
                        block_pos,
                        chunk_cords,
                        &chunk_map,
                        len,
                    );

                    // Add faces (uncull quads) facing the broken block from other chunks.
                    // This is only done in cube sub-chunks, because no other sub-chunk type
                    // requires culling & unculling.
                    if cube_chunk {
                        for (face, neighboring_block_pos) in
                            enumerate_neighbors_across_chunks(block_pos, CHUNK_DIMS)
                        {
                            let new_cords = IVec2::from(to_cords(Some(Direction::from(face))))
                                + IVec2::from(chunk_cords);
                            if let Ok((Grid(neighbor_chunk_grid), _, _, CubeChild(n_cube_chunk))) =
                                parent_chunks.get(
                                    *chunk_map
                                        .pos_to_ent
                                        .get(&new_cords)
                                        .unwrap_or(&Entity::PLACEHOLDER),
                                )
                            {
                                let (neighboring_metadata, _, _) =
                                    chunk_metadata.get(*n_cube_chunk).unwrap();
                                match &mut *neighboring_metadata.0.write().unwrap() {
                                    MetaData::CubeMD(ref mut metadata) => metadata.log(
                                        VoxelChange::AddFaces,
                                        neighboring_block_pos,
                                        neighbor_chunk_grid
                                            .read()
                                            .unwrap()
                                            .get_block(neighboring_block_pos)
                                            .unwrap(),
                                        [Some(Block::AIR); 6],
                                    ),
                                    _ => {}
                                }
                                commands.entity(*n_cube_chunk).insert(ToUpdate);
                            }
                        }
                    }
                }
            }
            // Send a world update event that a block has been broken.
            send_world_updates_surrounding_blocks(
                block_pos,
                chunk_cords,
                &mut world_block_update_sender,
                BlockUpdate::Broken,
            );
        }
    }
}
