use bevy_xpbd_3d::prelude::contact_query::contact;
use bevy_xpbd_3d::prelude::Collider;

use crate::blocks::BlockPropertyRegistry;
use crate::chunk::{chunkmd::CMMD, Chunk, ChunkCords, ChunkMap, Cords, Grid, ToUpdate, CHUNK_DIMS};
use crate::chunk::{ChunkChild, CubeChunk, XSpriteChunk};
use crate::prelude::notical;
use crate::utils::to_global_pos;

use super::blockreg::BlockRegistry;
use super::existence_conditions::ExistenceConditionSolverData;
use super::properties::DynamicProperty;
use super::*;

#[derive(Event)]
pub struct BlockPlaceEvent(pub Entity, pub usize, pub Face, pub Block);

#[derive(Event)]
pub struct PlaceBlockGlobalEvent {
    pub block: Block,
    pub chunk_pos: ChunkCords,
    pub block_index: usize,
}

pub(super) fn handle_place_block_event(
    mut place_block_event_reader: EventReader<BlockPlaceEvent>,
    mut global_block_place_event_sender: EventWriter<PlaceBlockGlobalEvent>,
    child_chunk_query: Query<&Parent, With<ChunkChild>>,
    dyn_preg: Res<BlockPropertyRegistry<DynamicProperty>>,
    parent_chunk_query: Query<(&Cords, &Grid)>,
    player_q: Query<(&Transform, &Collider), With<PhysicalPlayer>>,
    blocks_q: Query<(&Block, &Collider, &Transform)>,
) {
    'event_loop: for place_block_event in place_block_event_reader.read() {
        let BlockPlaceEvent(entity, index, face, block_to_place) = place_block_event;
        if let Ok(parent) = child_chunk_query.get(*entity) {
            if let Ok((Cords(cords), Grid(grid))) = parent_chunk_query.get(parent.get()) {
                let (block_index, chunk_pos) = {
                    if let Some(neighbor) = get_neighbor(*index, *face, CHUNK_DIMS) {
                        (neighbor, *cords)
                    } else {
                        let neighbor =
                            get_neigbhor_across_chunk_safe(CHUNK_DIMS, *index, *face).unwrap();
                        let change = to_cords(Some(notical::Direction::from(*face)));
                        let new_cords = [cords[0] + change[0], cords[1] + change[1]];
                        (neighbor, new_cords)
                    }
                };
                // The global positin of the block
                let block_global_pos =
                    to_global_pos(block_index, chunk_pos, VOXEL_DIMS.into(), CHUNK_DIMS);
                // check if the to-be placed block overlaps with the player
                if BlockPropertyRegistry::is_collidable(block_to_place) {
                    let (transform, collider) = player_q.get_single().unwrap();
                    if contact(
                        collider,
                        transform.translation,
                        Quat::IDENTITY,
                        &Collider::cuboid(0.99, 0.85, 0.99),
                        block_global_pos,
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
                            block_global_pos,
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
                let mut surrounding_blocks = [None; 6];
                (0..6).into_iter().for_each(|j| {
                    surrounding_blocks[j] = Some(
                        get_neighbor(block_index, Face::from(j), CHUNK_DIMS)
                            .map_or(Block::AIR, |i| grid.read().unwrap()[i]),
                    )
                });
                let solver_data = ExistenceConditionSolverData { surrounding_blocks };
                for dynamic_property in dyn_preg.get_properties(block_to_place) {
                    match dynamic_property {
                        DynamicProperty::ExistenceCondition(cond) => {
                            if !cond.solve(solver_data) {
                                info!("Attemp to place block that can't exist there was stopped");
                                continue 'event_loop;
                            }
                        }
                        _ => {}
                    }
                }

                // send the global block place event
                global_block_place_event_sender.send(PlaceBlockGlobalEvent {
                    block: *block_to_place,
                    chunk_pos,
                    block_index,
                });
            }
        }
    }
}

pub fn global_block_placer(
    mut global_block_place_events: EventReader<PlaceBlockGlobalEvent>,
    mut world_block_update_sender: EventWriter<WorldBlockUpdate>,
    mut break_block_global_sender: EventWriter<BreakBlockGlobalEvent>,
    mut commands: Commands,
    breg: Res<BlockRegistry>,
    chunk_map: Res<ChunkMap>,
    parent_chunks: Query<(&Grid, &Children), With<Chunk>>,
    chunk_metadata: Query<(&CMMD, Has<CubeChunk>, Has<XSpriteChunk>)>,
) {
    let len = global_block_place_events.len();
    for PlaceBlockGlobalEvent {
        block,
        chunk_pos,
        block_index: block_pos,
    } in global_block_place_events.read()
    {
        // Placing an Air block is equivalent to breaking the block
        if *block == Block::AIR {
            break_block_global_sender.send(BreakBlockGlobalEvent {
                chunk_pos: Some(*chunk_pos),
                chunk_entity: None,
                block_index: *block_pos,
            });
            continue;
        }
        if let (Some(Grid(grid)), children) =
            chunk_map
                .pos_to_ent
                .get(chunk_pos)
                .map_or((None, [Entity::PLACEHOLDER].iter()), |e| {
                    parent_chunks
                        .get(*e)
                        .map(|(g, c)| (Some(g), c.iter()))
                        .unwrap_or((None, [Entity::PLACEHOLDER].iter()))
                })
        {
            let adj_blocks = [None::<Option<i8>>; 6]
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    get_neighbor(*block_pos, Face::from(i), CHUNK_DIMS)
                        .map(|n| grid.read().unwrap()[n])
                })
                .collect::<Vec<Option<Block>>>()
                .try_into()
                .unwrap();

            for child in children {
                if let Ok((md, cube_chunk, xsprite_chunk)) = chunk_metadata.get(*child) {
                    // make sure we update the metadata of the right chunk
                    match breg.get_mesh(block) {
                        VoxelMesh::NormalCube(_) if xsprite_chunk => continue,
                        VoxelMesh::XSprite(_) if cube_chunk => continue,
                        VoxelMesh::Null => continue,
                        _ => {}
                    }
                    md.0.write()
                        .unwrap()
                        .log_place(*block_pos, *block, adj_blocks);

                    commands.entity(*child).insert(ToUpdate);
                    asl2ac(&mut commands, *block_pos, *chunk_pos, &chunk_map, len);
                }
            }

            grid.write().unwrap()[*block_pos] = *block;
            send_world_updates_surrounding_blocks(
                *block_pos,
                *chunk_pos,
                &mut world_block_update_sender,
                BlockUpdate::Placed,
            );
        }
    }
}
