use crate::chunk::{Chunk, ChunkCords, ChunkMap, Cords, Grid, ToUpdate, CHUNK_DIMS};
use crate::mesh_utils::chunkmd::CMMD;
use crate::mesh_utils::{ChunkChild, CubeChunk, XSpriteChunk};
use crate::prelude::notical;

use super::blockreg::BlockRegistry;
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
    parent_chunk_query: Query<&Cords>,
) {
    for BlockPlaceEvent(entity, index, face, block_to_place) in place_block_event_reader.read() {
        if let Ok(parent) = child_chunk_query.get(*entity) {
            if let Ok(Cords(cords)) = parent_chunk_query.get(parent.get()) {
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
                global_block_place_event_sender.send(PlaceBlockGlobalEvent {
                    block: *block_to_place,
                    chunk_pos,
                    block_index,
                });
            }
        }
    }
}

pub(super) fn global_block_placer(
    mut global_block_place_events: EventReader<PlaceBlockGlobalEvent>,
    mut world_block_update_sender: EventWriter<WorldBlockUpdate>,
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
                        VoxelMesh::CustomMesh(_) if cube_chunk => continue,
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
