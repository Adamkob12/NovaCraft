use super::*;
use crate::chunk::{
    chunkmd::{ChunkMD, CMMD},
    Chunk, ChunkMap, Cords, CubeChunk, Grid, MainChild, ToUpdate, CHUNK_DIMS,
};

#[derive(Event)]
pub struct BreakBlockGlobalEvent {
    pub chunk_pos: Option<ChunkCords>,
    pub chunk_entity: Option<Entity>,
    pub block_index: usize,
}

#[allow(dead_code)]
impl BreakBlockGlobalEvent {
    pub fn from_block_pos_safe(block_pos: Vec3) -> Option<Self> {
        let (cords, index, _) = position_to_chunk_position(block_pos, CHUNK_DIMS);
        let block_index = one_d_cords_safe(index, CHUNK_DIMS)?;
        Some(Self {
            chunk_pos: Some(cords),
            chunk_entity: None,
            block_index,
        })
    }

    pub fn from_block_pos(block_pos: Vec3) -> Self {
        let (cords, index, _) = position_to_chunk_position(block_pos, CHUNK_DIMS);
        let block_index = one_d_cords_safe(index, CHUNK_DIMS).unwrap();
        Self {
            chunk_pos: Some(cords),
            chunk_entity: None,
            block_index,
        }
    }

    pub fn with_chunk_entity(mut self, entity: Entity) -> Self {
        self.chunk_entity = Some(entity);
        self
    }

    pub fn new(block_index: usize) -> Self {
        Self {
            chunk_entity: None,
            block_index,
            chunk_pos: None,
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
            block_index,
            chunk_pos,
        } = global_block_break;
        if let Some(parent_chunk) = chunk_entity.map_or(
            chunk_pos.map(|cords| chunk_map.pos_to_ent.get(&cords).copied().unwrap()),
            |e| {
                chunk_metadata
                    .get(e)
                    .ok()
                    .map(|(_, parent, _)| parent.get())
            },
        ) {
            let (Grid(grid), &Cords(chunk_pos), children, _) =
                parent_chunks.get(parent_chunk).unwrap();
            grid.write().unwrap()[*block_index] = Block::AIR;
            let adj_blocks = [None::<Option<i8>>; 6]
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    get_neighbor(*block_index, Face::from(i), CHUNK_DIMS)
                        .map(|n| grid.read().unwrap()[n])
                })
                .collect::<Vec<Option<Block>>>()
                .try_into()
                .unwrap();
            for child in children {
                if let Ok((md, _, cube_chunk)) = chunk_metadata.get(*child) {
                    md.0.write().unwrap().log_break(*block_index, adj_blocks);

                    commands.entity(*child).insert(ToUpdate);
                    asl2ac(&mut commands, *block_index, chunk_pos, &chunk_map, len);

                    // Add faces (uncull quads) facing the broken block from other chunks.
                    if cube_chunk {
                        for (face, neighbor) in
                            get_neigbhors_from_across_chunks(CHUNK_DIMS, *block_index)
                        {
                            let new_cords = IVec2::from(to_cords(Some(Direction::from(face))))
                                + IVec2::from(chunk_pos);
                            let new_cords: [i32; 2] = new_cords.into();
                            if let Ok((Grid(n_grid), _, _, MainChild(n_cube_chunk))) = parent_chunks
                                .get(
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
                                        n_grid.read().unwrap()[neighbor],
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
                *block_index,
                chunk_pos,
                &mut world_block_update_sender,
                BlockUpdate::Broken,
            );
        }
    }
}
