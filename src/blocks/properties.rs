use novacraft_derive::InitBlockProperties;

use super::{dynamic_property::BlockTransformation, *};
const BLOCK_DENSITY: f32 = 100.0;

pub trait BlockProperty {
    fn get_property_type() -> BlockPropertyTypes;
}

#[allow(non_camel_case_types)]
#[derive(InitBlockProperties)]
pub enum BlockPropertyTypes {
    Physical(Option<PhysicalProperty>),
    PassiveProperty(Option<PassiveProperty>),
    Perceptible(Option<PerceptibleProperty>),
    Dynamic(Option<DynamicProperty>),
    ExCond(Option<ExistenceCondition>),
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
    BlockAbove(BlockTransformation),
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
