use crate::{chunk::ChunkCords, prelude::*};

mod block_defs;
mod block_descriptor;
pub mod dynamic_property;
pub mod existence_conditions;
#[allow(non_snake_case)]
pub mod init_blocks;
pub mod meshreg;
pub mod properties;
pub use init_blocks::*;

use existence_conditions::ExistenceCondition;

/// [`WorldBlockUpdates`](WorldBlockUpdate) are what powers the interaction between blocks. When a
/// block has been broken, placed or any of the variants of [`BlockUpdate`] it sends an update to
/// the surrounding blocks. For example:
/// Block breaks --> Sends update to block above --> if the block is affected by gravity, the game
/// handles that case accordingly.
///
/// A [`WorldBlockUpdate`] only needs a global position to be sent (block position, chunk cords).
/// An "Undefined" [`WorldBlockUpdate`] means an update where the [`block_update`](WorldBlockUpdate::block_update)
/// field is [`None`]. And vice versa.
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

    /// Set the [`WorldBlockUpdate`] type.
    pub fn with_block_update(mut self, block_update: BlockUpdate) -> Self {
        self.block_update = Some(block_update);
        self
    }
}

/// All of the possible types of [`WorldBlockUpdate`], basically any way a block can change.
pub enum BlockUpdate {
    Broken,
    Placed,
}

pub struct BlocksPlugin;

/// A collection of constants used to generate the meshes of voxels.
mod consts_to_generate_voxel_mesh {
    pub const VOXEL_DIMS: [f32; 3] = [1.0, 1.0, 1.0];
    pub const VOXEL_CENTER: [f32; 3] = [0.0, 0.0, 0.0];
    pub const TEXTURE_ATLAS_DIMS: [u32; 2] = [10, 10];
    pub const PADDING: f32 = 0.0;
    pub const COLOR_INTENSITY: f32 = 1.0;
    pub const ALPHA: f32 = 1.0;
    pub const XSPRITE_SCALE: f32 = 0.85;
}
pub(super) use consts_to_generate_voxel_mesh::*;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WorldBlockUpdate>();
        app.add_plugins(BlockRegistriesPlugin);
    }
}
