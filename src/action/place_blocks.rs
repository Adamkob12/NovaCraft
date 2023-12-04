// REFACTORED

use bevy_xpbd_3d::prelude::contact_query::contact;
use bevy_xpbd_3d::prelude::Collider;

use crate::blocks::BlockPropertyRegistry;
use crate::chunk::{
    chunkmd::SubChunkMD, ChunkCords, ChunkMap, Cords, Grid, ParentChunk, ToUpdate, CHUNK_DIMS,
};
use crate::chunk::{CubeSubChunk, Subchunk, XSpriteSubChunk};
use crate::prelude::notical;

use super::existence_conditions::ExistenceConditionSolverData;
use super::meshreg::MeshRegistry;
use super::properties::DynamicProperty;
use super::*;

/// An event that contains information about a *player* that has placed a block. Note this is
/// different that a block being broken by the environment, for example.
/// The parameters are (left to right):
///     -[Entity]: The target sub-chunk entity. When the player placed the block, which chunk was
///         he looking at.
///     -[BlockPos]: The position of the block that came in contact with the line of sight of the
///         player (The block at which the player was aiming at).
///     -[Face]: The face of the block that the player hit.
///     -[Block]: The block that the player was holding (the block to place).
///
#[derive(Event)]
pub struct BlockPlaceEvent(pub Entity, pub BlockPos, pub Face, pub Block);

/// The final event in the block-placing pipeline. The modular design of the pipeline
/// allows for this event to be called from all over the code. This event will be processed
/// by the `global_block_placer` system.
#[derive(Event)]
pub struct PlaceBlockGlobalEvent {
    pub block: Block,
    pub chunk_cords: ChunkCords,
    pub block_pos: BlockPos,
}

impl PlaceBlockGlobalEvent {
    pub fn from_global_pos(global_pos: BlockGlobalPos, block: Block) -> Self {
        Self {
            block,
            block_pos: global_pos.pos,
            chunk_cords: global_pos.chunk_cords,
        }
    }
}

/// This system handles `BlockPlaceEvent`(s). Those are events when the player placed a block.
/// There is specific logic in this system that prevents undesirable block placements, such as:
/// grass in the air, block that overlaps with player, block that overlaps with falling blocks,
/// etc. In the event where a block would be placed by the environment or by a command, this logic
/// will not be checked before placing the block.
pub(super) fn handle_place_block_event(
    mut place_block_event_reader: EventReader<BlockPlaceEvent>,
    mut global_block_place_event_sender: EventWriter<PlaceBlockGlobalEvent>,
    child_chunk_query: Query<&Parent, With<Subchunk>>,
    dyn_preg: Res<BlockPropertyRegistry<DynamicProperty>>,
    parent_chunk_query: Query<(&Cords, &Grid)>,
    player_q: Query<(&Transform, &Collider), With<PhysicalPlayer>>,
    blocks_q: Query<(&Block, &Collider, &Transform)>,
) {
    'event_loop: for place_block_event in place_block_event_reader.read() {
        let BlockPlaceEvent(subchunk_entity, block_pos, face, block_to_place) = place_block_event;
        if let Ok(parent) = child_chunk_query.get(*subchunk_entity) {
            if let Ok((Cords(chunk_cords), Grid(chunk_grid))) = parent_chunk_query.get(parent.get())
            {
                // Get the position and chunk cords of the actual place to put the block in. This
                // is calculated by seeing which side of the block the player was aiming at.
                let (block_pos, chunk_cords) = {
                    if let Some(neighbor) = neighbor_pos(*block_pos, *face, CHUNK_DIMS) {
                        (neighbor, *chunk_cords)
                    } else {
                        let neighbor =
                            neighbor_across_chunk(*block_pos, *face, CHUNK_DIMS).unwrap();
                        let change = to_cords(Some(notical::Direction::from(*face)));
                        let new_cords = [chunk_cords[0] + change[0], chunk_cords[1] + change[1]];
                        (neighbor, new_cords.into())
                    }
                };

                let global_pos = BlockGlobalPos::new(block_pos, chunk_cords);
                let block_translation =
                    global_block_pos_to_block_trans(global_pos, VOXEL_DIMS.into(), CHUNK_DIMS);

                // check if the to-be placed block overlaps with the player
                if BlockPropertyRegistry::is_collidable(block_to_place) {
                    let (transform, collider) = player_q.get_single().unwrap();
                    if contact(
                        collider,
                        transform.translation,
                        Quat::IDENTITY,
                        &Collider::cuboid(0.99, 0.85, 0.99),
                        block_translation,
                        Quat::IDENTITY,
                        0.0,
                    )
                    .unwrap()
                    .is_some()
                    {
                        info!("Attempt to place block that overlaps with player was stopped.");
                        continue 'event_loop;
                    }
                }

                // check if the to-be placed block overlaps with any current out-of-chunk blocks
                for (_block, collider, transform) in blocks_q.iter() {
                    // In the future, this might be a condition about the block itself.
                    if true {
                        if contact(
                            collider,
                            transform.translation,
                            Quat::IDENTITY,
                            &Collider::cuboid(0.99, 0.99, 0.99),
                            block_translation,
                            Quat::IDENTITY,
                            0.0,
                        )
                        .unwrap()
                        .is_some()
                        {
                            info!("Attempt to place block that overlaps with another block was stopped.");
                            continue 'event_loop;
                        }
                    }
                }

                // Check if the to-be placed block can even exist in the given place
                // (based on the defined DynamicProperty::ExistenceCondition)
                let solver_data = ExistenceConditionSolverData {
                    surrounding_blocks: chunk_grid.read().unwrap().get_neighbors(block_pos),
                };
                for dynamic_property in dyn_preg.get_properties(block_to_place) {
                    match dynamic_property {
                        DynamicProperty::ExistenceCondition(cond) => {
                            if !cond.solve(solver_data) {
                                info!("Attemp to place block in a position that it can't exist in was stopped");
                                continue 'event_loop;
                            }
                        }
                        _ => {}
                    }
                }

                // send the global block place event
                global_block_place_event_sender.send(PlaceBlockGlobalEvent {
                    block: *block_to_place,
                    chunk_cords,
                    block_pos,
                });
            }
        }
    }
}

/// This system executes once every frame. It is the final stage of the block placing pipeline, and after its
/// execution, block-placing is unreversable. It processes all of the pending `PlaceBlockGlobalEvent`(s)
/// and marks the chunks that need to be updated. The meshes those chunks will be updated thereafter.
pub fn global_block_placer(
    mut global_block_place_events: EventReader<PlaceBlockGlobalEvent>,
    mut world_block_update_sender: EventWriter<WorldBlockUpdate>,
    mut break_block_global_sender: EventWriter<BreakBlockGlobalEvent>,
    mut commands: Commands,
    mreg: Res<MeshRegistry>,
    chunk_map: Res<ChunkMap>,
    parent_chunks: Query<(&Grid, &Children), With<ParentChunk>>,
    chunk_metadata: Query<(&SubChunkMD, Has<CubeSubChunk>, Has<XSpriteSubChunk>)>,
) {
    let len = global_block_place_events.len();
    for &PlaceBlockGlobalEvent {
        block,
        chunk_cords,
        block_pos,
    } in global_block_place_events.read()
    {
        // Placing an Air block is equivalent to breaking the block, so we just send a block
        // breaking event to execute next frame.
        if block == Block::AIR {
            break_block_global_sender.send(BreakBlockGlobalEvent::from_global_pos(
                BlockGlobalPos::new(block_pos, chunk_cords),
            ));
            continue;
        }
        if let (Some(Grid(chunk_grid)), subchunks) = chunk_map.pos_to_ent.get(&chunk_cords).map_or(
            (None, [Entity::PLACEHOLDER].iter()),
            |e| {
                parent_chunks
                    .get(*e)
                    .map(|(g, c)| (Some(g), c.iter()))
                    .unwrap_or((None, [Entity::PLACEHOLDER].iter()))
            },
        ) {
            for subchunk in subchunks {
                if let Ok((subchunk_md, cube_chunk, xsprite_chunk)) = chunk_metadata.get(*subchunk)
                {
                    // make sure we update the metadata of the right subchunk
                    match mreg.get_mesh(&block) {
                        VoxelMesh::NormalCube(_) if xsprite_chunk => continue,
                        VoxelMesh::XSprite(_) if cube_chunk => continue,
                        VoxelMesh::Null => continue,
                        _ => {}
                    }
                    // Update the metadata
                    subchunk_md.0.write().unwrap().log_place(
                        block_pos,
                        block,
                        chunk_grid.read().unwrap().get_neighbors(block_pos),
                    );

                    // Insert marker components and apply smooth lighting.
                    commands.entity(*subchunk).insert(ToUpdate);
                    apply_smooth_lighting_util(
                        &mut commands,
                        block_pos,
                        chunk_cords,
                        &chunk_map,
                        len,
                    );
                }
            }

            // Set the new block in the grid, broadcast a world update.
            let _ = chunk_grid.write().unwrap().set_block(block, block_pos);
            send_world_updates_surrounding_blocks(
                block_pos,
                chunk_cords,
                &mut world_block_update_sender,
                BlockUpdate::Placed,
            );
        }
    }
}
