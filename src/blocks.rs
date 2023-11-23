use crate::{mesh_utils::ChunkCords, prelude::*};

mod block_descriptor;
pub mod blockreg;
pub mod existence_conditions;
pub mod properties;
mod registries;
mod xsprite_mesh;

use std::fmt;

use self::{
    blockreg::BlockRegistry, existence_conditions::ExistenceConditions,
    properties::BlockPropertyRegistry,
};

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

#[repr(u16)]
#[derive(Eq, PartialEq, Clone, Copy, Component)]
pub enum Block {
    AIR = 0,
    DIRT = 1,
    GRASS = 2,
    STONE = 3,
    GREENERY = 4,
    SAND = 5,
}

impl Into<&'static str> for Block {
    fn into(self) -> &'static str {
        match self {
            Self::AIR => "Air",
            Self::DIRT => "Dirt",
            Self::GRASS => "Grass",
            Self::STONE => "Stone",
            Self::GREENERY => "Greenery",
            Self::SAND => "Sand",
        }
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NovaCraft::Block::{}", Into::<&'static str>::into(*self))
    }
}

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WorldBlockUpdate>();
        app.init_resource::<BlockRegistry>()
            .init_resource::<BlockPropertyRegistry>()
            .init_resource::<ExistenceConditions>();
    }
}
