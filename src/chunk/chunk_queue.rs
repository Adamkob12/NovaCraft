use super::{ChunkCords, ChunkMap, CHUNK_TOTAL_BLOCKS, HEIGHT};
use crate::blocks::blockreg::BlockRegistry;
use crate::chunk::{Block, CHUNK_DIMS};
use crate::prelude::*;
use crate::terrain::generate_flat_chunk;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use noise::Perlin;
use std::sync::Arc;

#[derive(Resource, Default)]
pub struct ChunkQueue {
    // true = spawn, false= despawn
    queue: Vec<[i32; 2]>,
    pub panic_when_cant_find_chunk: bool,
}

#[derive(Component)]
pub struct ComputeChunk(
    pub Task<Option<((Mesh, MeshMD<Block>), [Block; CHUNK_TOTAL_BLOCKS], [i32; 2])>>,
);

impl ChunkQueue {
    pub fn enqueue(&mut self, cords: [i32; 2]) {
        self.queue.push(cords);
    }
    // Dequeue all the pending chunks to spawn / despawn.
    pub fn dequeue_all<F: Fn(&ChunkCords) -> bool>(
        &mut self,
        chunk_map: &mut ChunkMap,
        mut commands: Commands,
        breg: Arc<BlockRegistry>,
        condition: Option<F>,
    ) {
        if self.queue.is_empty() {
            return;
        }
        let thread_pool = AsyncComputeTaskPool::get();
        for cords in self.queue.as_slice() {
            let task;
            if chunk_map.pos_to_ent.contains_key(cords) {
                assert!(
                    !self.panic_when_cant_find_chunk,
                    "Can't spawn chunk that is already spawned."
                );
                continue;
            }

            let breg = Arc::clone(&breg);
            let cords = *cords;
            if let Some(ref condition) = condition {
                if !condition(&cords) {
                    continue;
                }
            }
            chunk_map.pos_to_ent.insert(cords, Entity::PLACEHOLDER);
            task = thread_pool.spawn(async move {
                let grid = generate_flat_chunk(HEIGHT / 2);
                let t = mesh_grid(
                    CHUNK_DIMS,
                    &[Bottom /* , Forward, Back, Right, Left */],
                    &grid,
                    breg.as_ref(),
                    MeshingAlgorithm::Culling,
                    Some(PbsParameters {
                        pbs_value: 0.51,
                        min: 0.2,
                        smoothing: PbsSmoothing::Low,
                    }),
                )?;
                Some((t, grid, cords))
            });
            commands.spawn(ComputeChunk(task));
        }

        self.queue.clear();
    }
}
