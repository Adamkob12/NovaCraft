use crate::blocks::Block;
use crate::chunk::{CHUNK_DIMS, CHUNK_TOTAL_BLOCKS, HEIGHT, LENGTH, WIDTH};
use bevy_meshem::prelude::one_d_cords;
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
pub const NOISE_SEED: usize = 10;
pub const NOISE_SEED_SQRD: usize = NOISE_SEED * NOISE_SEED;
pub const NOISE_FACTOR_CONT: f64 = 0.012;
pub const NOISE_FACTOR_SCALE: f64 = 2.1;

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

// Generate chunk from noise
pub fn generate_chunk(
    cords: [i32; 2],
    noise: &impl NoiseFn<f64, 2>,
) -> [Block; CHUNK_TOTAL_BLOCKS] {
    let mut rng = rand::thread_rng();
    let mut height_map: [usize; WIDTH * LENGTH] = [0; WIDTH * LENGTH];
    let mut chunk = [Block::AIR; CHUNK_TOTAL_BLOCKS];
    // First, generate a height map
    for j in 0..LENGTH {
        for i in 0..WIDTH {
            height_map[i + j * WIDTH] = (HEIGHT as f64 * 1.0 / NOISE_FACTOR_SCALE
                + (noise.get([
                    ((i as i32 + cords[0] * WIDTH as i32) as f64 + 0.5) * NOISE_FACTOR_CONT,
                    ((j as i32 + cords[1] * LENGTH as i32) as f64 + 0.5) * NOISE_FACTOR_CONT,
                ]) * HEIGHT as f64
                    * (1.0 - 1.0 / NOISE_FACTOR_SCALE)))
                .min(HEIGHT as f64 - 1.0) as usize;
        }
    }
    // From the height map, assign a value to each block based on wether it is below or above the
    // height level at that position, if it is the exact position of the height, grass block.
    for y in 0..HEIGHT {
        for z in 0..LENGTH {
            for x in 0..WIDTH {
                if height_map[x + z * WIDTH] == y && y > HEIGHT / 2 {
                    chunk[x + z * WIDTH + y * WIDTH * LENGTH] = Block::GRASS;
                } else if height_map[x + z * WIDTH] + 1 == y && y > HEIGHT / 2 + 1 {
                    chunk[x + z * WIDTH + y * WIDTH * LENGTH] = {
                        let r: f32 = rng.gen();
                        if r > 0.95 {
                            Block::GREENERY
                        } else {
                            Block::AIR
                        }
                    }
                } else if y > height_map[x + z * WIDTH] {
                    chunk[x + z * WIDTH + y * WIDTH * LENGTH] = Block::AIR;
                } else if y > HEIGHT / 4 {
                    chunk[x + z * WIDTH + y * WIDTH * LENGTH] = Block::DIRT;
                } else if y <= HEIGHT / 4 {
                    chunk[x + z * WIDTH + y * WIDTH * LENGTH] = Block::STONE;
                }
            }
        }
    }
    chunk
}
