use crate::blocks::Block;
use crate::chunk::{Chunk, ChunkCloseToPlayer, Cords, Grid, CHUNK_DIMS, CHUNK_TOTAL_BLOCKS};
use crate::prelude::*;
use bevy::ecs::event::{Events, ManualEventReader};
use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    pbr::{
        ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionQualityLevel,
        ScreenSpaceAmbientOcclusionSettings,
    },
    prelude::*,
    render::camera::TemporalJitter,
};

pub(crate) const CAGE_SIZE: usize = 7;
pub(crate) const HALF_CAGE_I: i32 = (CAGE_SIZE / 2) as i32;
pub const CAGE_LEN: usize = CAGE_SIZE * CAGE_SIZE * CAGE_SIZE;
pub(crate) const CAGE_DIMS: (usize, usize, usize) = (CAGE_SIZE, CAGE_SIZE, CAGE_SIZE);

#[derive(Component)]
pub struct Cage {
    pub blocks: [Block; CAGE_LEN],
}

pub fn update_cage(
    chunk_query: Query<(&Cords, &Grid), (With<ChunkCloseToPlayer>, With<Chunk>)>,
    mut player_query: Query<(&mut Cage, &Transform), Changed<Transform>>,
) {
    if let Ok((mut cage, tran)) = player_query.get_single_mut() {
        let pos = tran.translation;
        let current_block: Vec3 = [pos.x.round(), pos.y.round(), pos.z.round()].into();
        for x in -HALF_CAGE_I..=HALF_CAGE_I {
            for z in -HALF_CAGE_I..=HALF_CAGE_I {
                for y in -HALF_CAGE_I..=HALF_CAGE_I {
                    let cage_index_3d = [
                        (x + HALF_CAGE_I) as usize,
                        (y + HALF_CAGE_I) as usize,
                        (z + HALF_CAGE_I) as usize,
                    ];
                    if let Some(cage_index_1d) = one_d_cords_safe(cage_index_3d, CAGE_DIMS) {
                        let block_pos = current_block + Vec3::new(x as f32, y as f32, z as f32);
                        let (chunk_pos, block_pos, flag) =
                            position_to_chunk_position(block_pos, CHUNK_DIMS);

                        let block = if !flag {
                            Block::AIR
                        } else {
                            let mut r = Block::AIR;
                            for (cords, grid) in chunk_query.iter() {
                                if cords.0 == chunk_pos {
                                    if let Some(block_pos) = one_d_cords_safe(block_pos, CHUNK_DIMS)
                                    {
                                        r = grid.0[block_pos];
                                    }
                                }
                            }
                            r
                        };
                        cage.blocks[cage_index_1d] = block;
                        cage.blocks = [Block::AIR; CAGE_LEN];
                    }
                }
            }
        }
    }
}
