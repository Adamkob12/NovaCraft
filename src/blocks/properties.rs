// REFACTORED

use novacraft_derive::InitBlockProperties;

use super::{dynamic_property::BlockTransformation, *};
const BLOCK_DENSITY: f32 = 100.0;

/// This is a marker trait for all of the Properties that require their own registry.
pub trait BlockProperty {}

/// Here we declare all of the block propertes we want to use in the game. The derive macro ['InitBlockProperties'](InitBlockProperties)
/// automatically implements the [`BlockProperty`] trait for them, defines and implements [`Default`] for
/// their own [`BlockPropertyRegistry`] and initializes them as a resource in Bevy.
#[allow(non_camel_case_types)]
#[derive(InitBlockProperties)]
pub enum __InitProperties__ {
    Physical(PhysicalProperty),
    PassiveProperty(PassiveProperty),
    Perceptible(PerceptibleProperty),
    Dynamic(DynamicProperty),
}

#[derive(Component)]
pub struct FallingBlock {
    pub origin: BlockPos,
}

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
    AudioSource,
}

pub enum DynamicProperty {
    BlockTransformIf(ExistenceCondition, BlockTransformation),
    ExistenceCondition(ExistenceCondition),
}

// Add some of our own implementation for the macro-generated BlockPropertyRegistry<PhysicalProperty>
impl BlockPropertyRegistry<PhysicalProperty> {
    pub fn get_density(block: &Block) -> f32 {
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
