use super::*;
use crate::chunk::{
    chunkmd::{ChunkMD, CMMD},
    Chunk, ChunkMap, Cords, CubeChunk, Grid, MainChild, ToUpdate, CHUNK_DIMS,
};

#[derive(Event)]
pub struct BreakBlockGlobalEvent {
    pub chunk_cords: Option<ChunkCords>,
    pub chunk_entity: Option<Entity>,
    pub block_pos: BlockPos,
}

#[allow(dead_code)]
impl BreakBlockGlobalEvent {
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

    pub fn with_chunk_entity(mut self, entity: Entity) -> Self {
        self.chunk_entity = Some(entity);
        self
    }

    pub fn new(block_pos: BlockPos) -> Self {
        Self {
            chunk_entity: None,
            block_pos,
            chunk_cords: None,
        }
    }
}

pub fn global_block_breaker(
    mut global_block_break_events: EventReader<BreakBlockGlobalEvent>,
    mut world_block_update_sender: EventWriter<WorldBlockUpdate>,
    mut commands: Commands,
    chunk_map: Res<ChunkMap>,
    parent_chunks: Query<(&Grid, &Cords, &Children, &MainChild), With<Chunk>>,
    chunk_metadata: Query<(&CMMD, &Parent, Has<CubeChunk>)>,
) {
    let len = global_block_break_events.len();
    for global_block_break in global_block_break_events.read() {
        let BreakBlockGlobalEvent {
            chunk_entity,
            block_pos,
            chunk_cords,
        } = *global_block_break;
        if let Some(parent_chunk) = chunk_entity.map_or(
            chunk_cords.map(|cords| chunk_map.pos_to_ent.get(&cords).copied().unwrap()),
            |e| {
                chunk_metadata
                    .get(e)
                    .ok()
                    .map(|(_, parent, _)| parent.get())
            },
        ) {
            let (Grid(chunk_grid), &Cords(chunk_cords), children, _) =
                parent_chunks.get(parent_chunk).unwrap();
            let _ = chunk_grid.write().unwrap().set_block(Block::AIR, block_pos);

            for child in children {
                if let Ok((md, _, cube_chunk)) = chunk_metadata.get(*child) {
                    md.0.write().unwrap().log_break(
                        block_pos,
                        chunk_grid.read().unwrap().get_neighbors(block_pos),
                    );

                    commands.entity(*child).insert(ToUpdate);
                    asl2ac(&mut commands, block_pos, chunk_cords, &chunk_map, len);

                    // Add faces (uncull quads) facing the broken block from other chunks.
                    if cube_chunk {
                        for (face, neighbor) in
                            enumerate_neighbors_across_chunks(block_pos, CHUNK_DIMS)
                        {
                            let new_cords = IVec2::from(to_cords(Some(Direction::from(face))))
                                + IVec2::from(chunk_cords);
                            if let Ok((Grid(neighbor_chunk_grid), _, _, MainChild(n_cube_chunk))) =
                                parent_chunks.get(
                                    *chunk_map
                                        .pos_to_ent
                                        .get(&new_cords)
                                        .unwrap_or(&Entity::PLACEHOLDER),
                                )
                            {
                                let (n_md, _, _) = chunk_metadata.get(*n_cube_chunk).unwrap();
                                match &mut *n_md.0.write().unwrap() {
                                    ChunkMD::CubeMD(ref mut metadata) => metadata.log(
                                        VoxelChange::AddFaces,
                                        neighbor,
                                        neighbor_chunk_grid
                                            .read()
                                            .unwrap()
                                            .get_block(neighbor)
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
            send_world_updates_surrounding_blocks(
                block_pos,
                chunk_cords,
                &mut world_block_update_sender,
                BlockUpdate::Broken,
            );
        }
    }
}
