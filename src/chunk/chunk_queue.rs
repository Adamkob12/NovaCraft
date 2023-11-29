use super::{ChunkCords, ChunkMap, RenderSettings, CHUNK_TOTAL_BLOCKS};
use crate::blocks::blockreg::BlockRegistry;
use crate::chunk::{Block, CHUNK_DIMS};
use crate::prelude::*;
use crate::terrain::{generate_chunk, TerrainConfig};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use noise::Perlin;
use std::sync::Arc;

const NOISE_SEED: u32 = 6175;

#[derive(Resource, Default)]
pub struct ChunkQueue {
    // true = spawn, false= despawn
    queue: Vec<[i32; 2]>,
    pub panic_when_cant_find_chunk: bool,
}

#[derive(Component)]
pub struct ComputeChunk(
    pub  Task<
        Option<(
            (Mesh, MeshMD<Block>),
            [Block; CHUNK_TOTAL_BLOCKS],
            [i32; 2],
            (Mesh, XSpriteMetaData<Block>),
        )>,
    >,
);

impl ChunkQueue {
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn enqueue(&mut self, cords: [i32; 2]) {
        self.queue.push(cords);
    }

    // Dequeue all the pending chunks to spawn / despawn.
    pub fn dequeue_all<F: Fn(&ChunkCords) -> bool>(
        &mut self,
        chunk_map: &mut ChunkMap,
        mut commands: Commands,
        breg: &Arc<BlockRegistry>,
        condition: Option<F>,
        render_settings: &RenderSettings,
        terrain_config: &TerrainConfig,
    ) {
        if self.queue.is_empty() {
            return;
        }
        let noise = Perlin::new(NOISE_SEED);
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

            let cords = *cords;
            if let Some(ref condition) = condition {
                if !condition(&cords) {
                    continue;
                }
            }
            let breg = Arc::clone(breg);
            chunk_map.pos_to_ent.insert(cords, Entity::PLACEHOLDER);
            let sl = render_settings.sl;
            let noise_factor_cont = terrain_config.noise_factor_cont;
            let noise_factor_scale = terrain_config.noise_factor_scale;
            task = thread_pool.spawn(async move {
                // let grid = generate_flat_chunk(HEIGHT / 2);
                let grid = generate_chunk(cords, &noise, noise_factor_cont, noise_factor_scale);
                let t = mesh_grid(
                    CHUNK_DIMS,
                    &[Bottom /* , Forward, Back, Right, Left */],
                    &grid,
                    breg.as_ref(),
                    MeshingAlgorithm::Culling,
                    sl,
                )?;
                let custom_voxel_meshes = meshify_xsprite_voxels(breg.as_ref(), &grid, CHUNK_DIMS);
                Some((t, grid, cords, custom_voxel_meshes))
            });
            commands.spawn(ComputeChunk(task));
        }

        self.queue.clear();
    }
}
