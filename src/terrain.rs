use crate::blocks::Block;
use crate::chunk::{CHUNK_DIMS, CHUNK_TOTAL_BLOCKS, HEIGHT, LENGTH, WIDTH};
use bevy_meshem::prelude::one_d_cords;
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
pub const NOISE_SEED: usize = 10;
pub const NOISE_SEED_SQRD: usize = NOISE_SEED * NOISE_SEED;
pub const NOISE_FACTOR_CONT: f64 = 0.020;
pub const NOISE_FACTOR_SCALE: f64 = 1.8;

// Generate chunk from noise
pub fn generate_flat_chunk(sea_level: usize) -> [Block; CHUNK_TOTAL_BLOCKS] {
    let mut rng = rand::thread_rng();
    let sea_level = sea_level.min(HEIGHT - 1);
    let mut chunk = [Block::AIR; CHUNK_TOTAL_BLOCKS];
    for k in 0..HEIGHT {
        for j in 0..LENGTH {
            for i in 0..WIDTH {
                chunk[one_d_cords([i, k, j], CHUNK_DIMS)] = {
                    if k == sea_level + 1 {
                        let r: f32 = rng.gen();
                        if r > 0.9 {
                            Block::GREENERY
                        } else {
                            Block::AIR
                        }
                    } else if k == sea_level {
                        Block::GRASS
                    } else if k < sea_level && k > sea_level - 3 {
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
