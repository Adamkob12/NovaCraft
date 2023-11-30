use novacraft_derive::InitBlockProperties;

use super::{dynamic_property::BlockTransformation, *};
const BLOCK_DENSITY: f32 = 100.0;

pub trait BlockProperty {}

#[allow(non_camel_case_types)]
#[derive(InitBlockProperties)]
pub enum PROPERTY_INITIALIZER {
    Physical(PhysicalProperty),
    PassiveProperty(PassiveProperty),
    Perceptible(PerceptibleProperty),
    Dynamic(DynamicProperty),
}

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

pub enum DynamicProperty {
    BlockTransformIf(ExistenceCondition, BlockTransformation),
    ExistenceCondition(ExistenceCondition),
}

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
            Block::GREENERY => false,
            _ => true,
        }
    }
}
