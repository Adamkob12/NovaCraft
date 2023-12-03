use crate::{chunk::ChunkCords, prelude::*};

#[allow(non_snake_case)]
pub mod INIT_BLOCKS;
mod block_defs;
mod block_descriptor;
pub mod dynamic_property;
pub mod existence_conditions;
pub mod meshreg;
pub mod properties;

pub use INIT_BLOCKS::*;

use existence_conditions::ExistenceCondition;

pub type BlockId = u16;

#[derive(Event)]
pub struct WorldBlockUpdate {
    pub chunk_cords: ChunkCords,
    pub block_pos: BlockPos,
    pub block_update: Option<BlockUpdate>,
}

impl WorldBlockUpdate {
    pub fn from_global_pos(global_pos: BlockGlobalPos) -> Self {
        Self {
            chunk_cords: global_pos.chunk_cords,
            block_pos: global_pos.pos,
            block_update: None,
        }
    }

    pub fn with_block_update(mut self, block_update: BlockUpdate) -> Self {
        self.block_update = Some(block_update);
        self
    }
}

pub enum BlockUpdate {
    Broken,
    Placed,
}

pub struct BlocksPlugin;

pub const VOXEL_DIMS: [f32; 3] = [1.0, 1.0, 1.0];
pub(super) const VOXEL_CENTER: [f32; 3] = [0.0, 0.0, 0.0];
pub(super) const TEXTURE_ATLAS_DIMS: [u32; 2] = [10, 10];
pub(super) const PADDING: f32 = 0.0;
pub(super) const COLOR_INTENSITY: f32 = 1.0;
pub(super) const ALPHA: f32 = 1.0;
pub(super) const XSPRITE_SCALE: f32 = 0.85;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WorldBlockUpdate>();
        app.add_plugins(BlockRegistriesPlugin);
    }
}
