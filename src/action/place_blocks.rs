use crate::chunk::{
    Chunk, ChunkCords, ChunkMap, Cords, Grid, MainChild, MainCulledMesh, ToUpdate, XSpriteChild,
    CHUNK_DIMS,
};
use crate::mesh_utils::{ChunkChild, XSpriteMesh};
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
                })
            }
        }
    }
}

pub(super) fn global_block_placer(
    mut global_block_place_events: EventReader<PlaceBlockGlobalEvent>,
    mut commands: Commands,
    chunk_map: Res<ChunkMap>,
    breg: Res<BlockRegistry>,
    parent_chunks: Query<(&Grid, &MainChild, &XSpriteChild), With<Chunk>>,
    cube_chunks: Query<&MainCulledMesh>,
    xsprite_chunks: Query<&XSpriteMesh>,
) {
    let len = global_block_place_events.len();
    for PlaceBlockGlobalEvent {
        block,
        chunk_pos,
        block_index: block_pos,
    } in global_block_place_events.read()
    {
        if let (Some(grid), cube_child, xsprite_child) = chunk_map.pos_to_ent.get(chunk_pos).map_or(
            (None, Entity::PLACEHOLDER, Entity::PLACEHOLDER),
            |e| {
                parent_chunks
                    .get(*e)
                    .map(|(grid, main, xsprite)| (Some(&grid.0), main.0, xsprite.0))
                    .unwrap_or((None, Entity::PLACEHOLDER, Entity::PLACEHOLDER))
            },
        ) {
            let mut adj_blocks = [None; 6];
            for i in 0..6 {
                let face = Face::from(i);
                if let Some(tmp) = get_neighbor(*block_pos, face, CHUNK_DIMS) {
                    adj_blocks[i] = Some(grid.read().unwrap()[tmp]);
                }
            }
            match breg.get_mesh(block) {
                VoxelMesh::Null => continue,
                VoxelMesh::NormalCube(_) => {
                    let MainCulledMesh(md) = cube_chunks.get(cube_child).unwrap();
                    let mut md = md.write().unwrap();
                    grid.write().unwrap()[*block_pos] = *block;
                    md.log(VoxelChange::Added, *block_pos, *block, adj_blocks);
                    commands.entity(cube_child).insert(ToUpdate);
                    asl2ac(&mut commands, *block_pos, *chunk_pos, &chunk_map, len);
                }
                VoxelMesh::CustomMesh(_) => {
                    let XSpriteMesh(md) = xsprite_chunks.get(xsprite_child).unwrap();
                    let mut md = md.write().unwrap();
                    grid.write().unwrap()[*block_pos] = *block;
                    md.log.push((VoxelChange::Added, *block, *block_pos));
                    commands.entity(xsprite_child).insert(ToUpdate);
                    asl2ac(&mut commands, *block_pos, *chunk_pos, &chunk_map, len);
                }
            }
        }
    }
}
