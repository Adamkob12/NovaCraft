use crate::mesh_utils::{
    ChunkCords, ChunkMap, Cords, Grid, MainChild, MainCulledMesh, ToApplySL, ToUpdate, XSpriteMesh,
    CHUNK_DIMS, CHUNK_TOTAL_BLOCKS, LENGTH, WIDTH,
};
use crate::prelude::notical;

use super::*;

#[derive(Event)]
pub struct BlockBreakEvent(pub Entity, pub usize);

#[derive(Event)]
pub struct BlockPlaceEvent(pub Entity, pub usize, pub Face);

pub(super) fn handle_break_block_event(
    mut commands: Commands,
    mut break_block_event_reader: EventReader<BlockBreakEvent>,
    chunk_map: Res<ChunkMap>,
    child_chunk_query: Query<(&MainCulledMesh, &Parent)>,
    parent_chunk_query: Query<(&Grid, &Cords, &MainChild)>,
) {
    let len = break_block_event_reader.len();
    for BlockBreakEvent(break_entity, break_index) in break_block_event_reader.read() {
        if let Ok((MainCulledMesh(metadata), parent)) = child_chunk_query.get(*break_entity) {
            if let Ok((Grid(grid), Cords(cords), _)) = parent_chunk_query.get(parent.get()) {
                let mut neighboring_blocks = [None; 6];
                let this_grid = grid.read().unwrap();
                let block_to_break = this_grid[*break_index];
                for i in 0..6 {
                    let face = Face::from(i);
                    if let Some(neighbor) = get_neighbor(*break_index, face, CHUNK_DIMS) {
                        neighboring_blocks[i] = Some(this_grid[neighbor])
                    } else {
                        let neighbor_index =
                            get_neigbhor_across_chunk_safe(CHUNK_DIMS, *break_index, face);
                        if neighbor_index.is_none() {
                            continue;
                        }
                        let neighbor_index = neighbor_index.unwrap();
                        let change = to_cords(Some(notical::Direction::from(face)));
                        let new_cords = [cords[0] + change[0], cords[1] + change[1]];
                        if let Some(neighboring_entity) = chunk_map.pos_to_ent.get(&new_cords) {
                            if let Ok((Grid(neighboring_grid), _, MainChild(child))) =
                                parent_chunk_query.get(*neighboring_entity)
                            {
                                if let Ok((MainCulledMesh(neighboring_metadata), _)) =
                                    child_chunk_query.get(*child)
                                {
                                    let nei_voxel =
                                        neighboring_grid.read().unwrap()[neighbor_index];
                                    neighboring_metadata.write().unwrap().log(
                                        VoxelChange::AddFaces,
                                        neighbor_index,
                                        nei_voxel,
                                        [Some(Block::AIR); 6],
                                    );
                                    commands.entity(*child).insert(ToUpdate);
                                }
                            }
                        }
                    }
                }
                drop(this_grid);
                grid.write().unwrap()[*break_index] = Block::AIR;
                metadata.write().unwrap().log(
                    VoxelChange::Broken,
                    *break_index,
                    block_to_break,
                    neighboring_blocks,
                );
                commands.entity(*break_entity).insert(ToUpdate);
                insert_apply_sl_to_adjacent_chunks(
                    &mut commands,
                    *break_index,
                    *cords,
                    chunk_map.as_ref(),
                    len,
                );
            }
        }
    }
}

pub(super) fn handle_place_block_event(
    mut commands: Commands,
    mut place_block_event_reader: EventReader<BlockPlaceEvent>,
    chunk_map: Res<ChunkMap>,
    child_chunk_query: Query<(&MainCulledMesh, &Parent)>,
    parent_chunk_query: Query<(&Grid, &Cords, &MainChild)>,
) {
    let len = place_block_event_reader.len();
    for BlockPlaceEvent(entity, index, face) in place_block_event_reader.read() {
        if let Ok((MainCulledMesh(metadata), parent)) = child_chunk_query.get(*entity) {
            if let Ok((Grid(grid), Cords(cords), _)) = parent_chunk_query.get(parent.get()) {
                let mut neighboring_blocks: [Option<Block>; 6] = [None; 6];
                if let Some(neighbor) = get_neighbor(*index, *face, CHUNK_DIMS) {
                    for i in 0..6 {
                        let f = Face::from(i);
                        if let Some(tmp) = get_neighbor(neighbor, f, CHUNK_DIMS) {
                            neighboring_blocks[i] = Some(grid.read().unwrap()[tmp]);
                        }
                    }
                    grid.write().unwrap()[neighbor] = Block::STONE;
                    metadata.write().unwrap().log(
                        VoxelChange::Added,
                        neighbor,
                        Block::STONE,
                        neighboring_blocks,
                    );
                    commands.entity(*entity).insert(ToUpdate);
                    insert_apply_sl_to_adjacent_chunks(
                        &mut commands,
                        neighbor,
                        *cords,
                        chunk_map.as_ref(),
                        len,
                    );
                } else {
                    let neighbor =
                        get_neigbhor_across_chunk_safe(CHUNK_DIMS, *index, *face).unwrap();
                    let change = to_cords(Some(notical::Direction::from(*face)));
                    let new_cords = [cords[0] + change[0], cords[1] + change[1]];

                    if let Some(neighboring_entity) = chunk_map.pos_to_ent.get(&new_cords) {
                        if let Ok((Grid(neighboring_grid), Cords(cords), MainChild(child))) =
                            parent_chunk_query.get(*neighboring_entity)
                        {
                            if let Ok((MainCulledMesh(neighboring_metadata), _)) =
                                child_chunk_query.get(*child)
                            {
                                for i in 0..6 {
                                    let f = Face::from(i);
                                    if let Some(tmp) = get_neighbor(neighbor, f, CHUNK_DIMS) {
                                        neighboring_blocks[i] =
                                            Some(neighboring_grid.read().unwrap()[tmp]);
                                    }
                                }
                                neighboring_grid.write().unwrap()[neighbor] = Block::STONE;
                                neighboring_metadata.write().unwrap().log(
                                    VoxelChange::Added,
                                    neighbor,
                                    Block::STONE,
                                    neighboring_blocks,
                                );
                                commands.entity(*child).insert(ToUpdate);
                                insert_apply_sl_to_adjacent_chunks(
                                    &mut commands,
                                    neighbor,
                                    *cords,
                                    chunk_map.as_ref(),
                                    len,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn insert_apply_sl_to_adjacent_chunks(
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

pub(super) fn handle_break_block_event_xsprite_chunk(
    mut commands: Commands,
    mut break_block_event_reader: EventReader<BlockBreakEvent>,
    child_chunk_query: Query<(&XSpriteMesh, &Parent)>,
    parent_chunk_query: Query<&Grid>,
) {
    for BlockBreakEvent(break_entity, break_index) in break_block_event_reader.read() {
        if let Ok((XSpriteMesh(metadata), parent)) = child_chunk_query.get(*break_entity) {
            if let Ok(Grid(grid)) = parent_chunk_query.get(parent.get()) {
                let this_grid = grid.read().unwrap();
                let block_to_break = this_grid[*break_index];
                metadata.write().unwrap().log.push((
                    VoxelChange::Broken,
                    block_to_break,
                    *break_index,
                ));
                drop(this_grid);
                grid.write().unwrap()[*break_index] = Block::AIR;
                commands.entity(*break_entity).insert(ToUpdate);
            }
        }
    }
}
