use crate::blocks::Block;
use crate::chunk::{ChunkCords, CHUNK_DIMS, CHUNK_TOTAL_BLOCKS_USIZE, HEIGHT, LENGTH, WIDTH};
use noise::NoiseFn;
use novacraft_utils::pos_to_index;
use rand::prelude::*;
pub const NOISE_SEED: usize = 10;
pub const NOISE_SEED_SQRD: usize = NOISE_SEED * NOISE_SEED;
pub const NOISE_FACTOR_CONT: f64 = 0.014;
pub const NOISE_FACTOR_SCALE: f64 = 1.7;

use crate::prelude::{Plugin, Resource};
#[derive(Resource)]
pub struct TerrainConfig {
    pub noise_seed: usize,
    pub noise_seed_sqrd: usize,
    pub noise_factor_cont: f64,
    pub noise_factor_scale: f64,
}
impl Default for TerrainConfig {
    fn default() -> Self {
        TerrainConfig {
            noise_seed: NOISE_SEED,
            noise_seed_sqrd: NOISE_SEED_SQRD,
            noise_factor_cont: NOISE_FACTOR_CONT,
            noise_factor_scale: NOISE_FACTOR_SCALE,
        }
    }
}

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<TerrainConfig>();
    }
}

// Generate chunk from noise
pub fn generate_flat_chunk(sea_level: u32) -> [Block; CHUNK_TOTAL_BLOCKS_USIZE] {
    let sea_level = sea_level.min(HEIGHT - 1);
    let mut chunk = [Block::AIR; CHUNK_TOTAL_BLOCKS_USIZE];
    for k in 0..HEIGHT {
        for j in 0..LENGTH {
            for i in 0..WIDTH {
                chunk[pos_to_index([i, k, j].into(), CHUNK_DIMS).unwrap() as usize] = {
                    if k == sea_level + 1 {
                        if true {
                            Block::GREENERY
                        } else {
                            Block::AIR
                        }
                    } else if k == sea_level {
                        Block::GRASS
                    } else if k < sea_level && k + 3 > sea_level {
                        Block::DIRT
                    } else if k < sea_level {
                        Block::STONE
                    } else {
                        Block::AIR
                    }
                }
            }
        }
    }
    chunk
}

// Generate chunk from noise
pub fn generate_chunk(
    cords: ChunkCords,
    noise: &impl NoiseFn<f64, 2>,
    noise_factor_cont: f64,
    noise_factor_scale: f64,
) -> [Block; CHUNK_TOTAL_BLOCKS_USIZE] {
    let mut rng = rand::thread_rng();
    let mut height_map: [u32; (WIDTH * LENGTH) as usize] = [0; (WIDTH * LENGTH) as usize];
    let mut chunk = [Block::AIR; CHUNK_TOTAL_BLOCKS_USIZE];
    // First, generate a height map
    for j in 0..LENGTH {
        for i in 0..WIDTH {
            height_map[(i + j * WIDTH) as usize] = (HEIGHT as f64 * 1.0 / noise_factor_scale
                + (noise.get([
                    ((i as i32 + cords[0] * WIDTH as i32) as f64 + 0.5) * noise_factor_cont,
                    ((j as i32 + cords[1] * LENGTH as i32) as f64 + 0.5) * noise_factor_cont,
                ]) * HEIGHT as f64
                    * (1.0 - 1.0 / noise_factor_scale)))
                .min(HEIGHT as f64 - 1.0) as u32;
        }
    }
    // From the height map, assign a value to each block based on wether it is below or above the
    // height level at that position, if it is the exact position of the height, grass block.
    for y in 0..HEIGHT {
        for z in 0..LENGTH {
            for x in 0..WIDTH {
                if height_map[(x + z * WIDTH) as usize] == y && y > HEIGHT / 2 {
                    chunk[(x + z * WIDTH + y * WIDTH * LENGTH) as usize] = Block::GRASS;
                } else if height_map[(x + z * WIDTH) as usize] + 1 == y && y > HEIGHT / 2 + 1 {
                    chunk[(x + z * WIDTH + y * WIDTH * LENGTH) as usize] = {
                        let r: f32 = rng.gen();
                        if r > 0.93 {
                            Block::GREENERY
                        } else {
                            Block::AIR
                        }
                    }
                } else if y > height_map[(x + z * WIDTH) as usize] {
                    chunk[(x + z * WIDTH + y * WIDTH * LENGTH) as usize] = Block::AIR;
                } else if y > HEIGHT / 4 {
                    chunk[(x + z * WIDTH + y * WIDTH * LENGTH) as usize] = Block::DIRT;
                } else if y <= HEIGHT / 4 {
                    chunk[(x + z * WIDTH + y * WIDTH * LENGTH) as usize] = Block::STONE;
                }
            }
        }
    }
    chunk
}
