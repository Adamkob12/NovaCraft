use super::block_descriptor::*;
use super::properties::*;
use novacraft_derive::InitBlocks;

#[allow(non_snake_case, non_camel_case_types)]
#[derive(InitBlocks)]
pub enum BLOCKS_INITIALIZER {
    Air(
        BlockDescriptor,
        PhysicalProperty,
        PassiveProperty,
        PerceptibleProperty,
        DynamicProperty,
        ExistenceCondition,
    ),
    Dirt(
        BlockDescriptor,
        PhysicalProperty,
        PassiveProperty,
        PerceptibleProperty,
        DynamicProperty,
        ExistenceCondition,
    ),
    Grass(
        BlockDescriptor,
        PhysicalProperty,
        PassiveProperty,
        PerceptibleProperty,
        DynamicProperty,
        ExistenceCondition,
    ),
    Stone(
        BlockDescriptor,
        PhysicalProperty,
        PassiveProperty,
        PerceptibleProperty,
        DynamicProperty,
        ExistenceCondition,
    ),
    Greenery(
        BlockDescriptor,
        PhysicalProperty,
        PassiveProperty,
        PerceptibleProperty,
        DynamicProperty,
        ExistenceCondition,
    ),
    Sand(
        BlockDescriptor,
        PhysicalProperty,
        PassiveProperty,
        PerceptibleProperty,
        DynamicProperty,
        ExistenceCondition,
    ),
}
