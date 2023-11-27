use crate::{mesh_utils::ChunkCords, prelude::*};

#[allow(non_snake_case)]
pub mod INIT_BLOCKS;
mod block_defs;
mod block_descriptor;
pub mod blockreg;
pub mod dynamic_property;
pub mod existence_conditions;
pub mod properties;
mod xsprite_mesh;

pub use INIT_BLOCKS::*;

use self::{blockreg::BlockRegistry, existence_conditions::ExistenceConditions, properties::*};

pub type BlockId = u16;

#[derive(Event)]
pub struct WorldBlockUpdate {
    pub chunk_pos: ChunkCords,
    pub block_index: usize,
    pub block_update: Option<BlockUpdate>,
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
pub(super) const GREENERY_SCALE: f32 = 0.85;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WorldBlockUpdate>();
        app.init_resource::<BlockRegistry>()
            .init_resource::<BlockPropertyRegistry<PassiveProperty>>()
            .init_resource::<BlockPropertyRegistry<PhysicalProperty>>()
            .init_resource::<BlockPropertyRegistry<PerceptibleProperty>>()
            .init_resource::<ExistenceConditions>();
    }
}
