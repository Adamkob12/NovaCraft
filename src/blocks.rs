pub mod blockreg;
mod id;
mod properties;
mod xsprite_mesh;

use std::fmt;

pub type BlockId = u16;

#[repr(u16)]
#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Block {
    AIR = 0,
    DIRT = 1,
    GRASS = 2,
    STONE = 3,
}

impl Into<&'static str> for Block {
    fn into(self) -> &'static str {
        match self {
            Self::AIR => "Air",
            Self::DIRT => "Dirt",
            Self::GRASS => "Grass",
            Self::STONE => "Stone",
        }
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NovaCraft::Block::{}", Into::<&'static str>::into(*self))
    }
}
