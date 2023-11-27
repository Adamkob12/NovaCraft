use super::*;
const BLOCK_DENSITY: f32 = 100.0;

pub trait Property {}

#[derive(Component)]
pub struct FallingBlock;

#[derive(Clone, Copy, PartialEq)]
pub enum PhysicalProperty {
    AffectedByGravity,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PassiveProperty {
    YieldToFallingBlock,
}

#[derive(Clone, Copy)]
pub enum PerceptibleProperty {
    LightSource(PointLight),
    // TODO: audio stuff
    AudioSource,
}

impl Property for PerceptibleProperty {}
impl Property for PassiveProperty {}
impl Property for PhysicalProperty {}

impl BlockPropertyRegistry<PhysicalProperty> {
    pub fn get_density(&self, block: &Block) -> f32 {
        match block {
            Block::AIR => 0.0,
            _ => BLOCK_DENSITY,
        }
    }

    pub fn is_collidable(block: &Block) -> bool {
        match block {
            Block::AIR => false,
            Block::DIRT => true,
            Block::GRASS => true,
            Block::STONE => true,
            Block::GREENERY => false,
            Block::SAND => true,
        }
    }
}
