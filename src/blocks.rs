pub mod blockreg;
mod id;
mod properties;
mod xsprite_mesh;

use std::fmt;

pub type BlockId = u16;

pub(super) const VOXEL_DIMS: [f32; 3] = [1.0, 1.0, 1.0];
pub(super) const VOXEL_CENTER: [f32; 3] = [0.0, 0.0, 0.0];
pub(super) const TEXTURE_ATLAS_DIMS: [u32; 2] = [10, 10];
pub(super) const PADDING: f32 = 1.0 / 16.0;
pub(super) const COLOR_INTENSITY: f32 = 1.0;
pub(super) const ALPHA: f32 = 1.0;
pub(super) const GREENERY_SCALE: f32 = 0.85;

#[repr(u16)]
#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Block {
    AIR = 0,
    DIRT = 1,
    GRASS = 2,
    STONE = 3,
    GREENERY = 4,
}

impl Into<&'static str> for Block {
    fn into(self) -> &'static str {
        match self {
            Self::AIR => "Air",
            Self::DIRT => "Dirt",
            Self::GRASS => "Grass",
            Self::STONE => "Stone",
            Self::GREENERY => "Greenery",
        }
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NovaCraft::Block::{}", Into::<&'static str>::into(*self))
    }
}
