use super::*;
use crate::mesh_utils::{
    ChunkMap, Cords, Grid, MainChild, MainCulledMesh, ToUpdate, XSpriteMesh, CHUNK_DIMS,
};
use crate::prelude::notical;

#[derive(Event)]
pub struct BlockBreakEvent(pub Entity, pub usize);

pub(super) fn handle_break_block_event(
    mut commands: Commands,
    mut break_block_event_reader: EventReader<BlockBreakEvent>,
    mut world_block_update_sender: EventWriter<WorldBlockUpdate>,
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
                asl2ac(&mut commands, *break_index, *cords, chunk_map.as_ref(), len);
                send_world_updates_surrounding_blocks(
                    *break_index,
                    *cords,
                    &mut world_block_update_sender,
                );
            }
        }
    }
}

pub(super) fn handle_break_block_event_xsprite_chunk(
    mut commands: Commands,
    mut break_block_event_reader: EventReader<BlockBreakEvent>,
    child_chunk_query: Query<(&XSpriteMesh, &Parent)>,
    parent_chunk_query: Query<(&Grid, &Cords)>,
    mut world_block_update_sender: EventWriter<WorldBlockUpdate>,
) {
    for BlockBreakEvent(break_entity, break_index) in break_block_event_reader.read() {
        if let Ok((XSpriteMesh(metadata), parent)) = child_chunk_query.get(*break_entity) {
            if let Ok((Grid(grid), Cords(cords))) = parent_chunk_query.get(parent.get()) {
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
                send_world_updates_surrounding_blocks(
                    *break_index,
                    *cords,
                    &mut world_block_update_sender,
                );
            }
        }
    }
}
