use super::block_descriptor::*;
use super::properties::*;
use super::ExistenceCondition;
use novacraft_derive::InitBlocks;

#[allow(non_snake_case, non_camel_case_types)]
#[derive(InitBlocks)]
pub enum BLOCKS_INITIALIZER {
    Air {
        block_descriptor: BlockDescriptor,
        physical: PhysicalProperty,
        passive: PassiveProperty,
        perceptible: PerceptibleProperty,
        dynamic: DynamicProperty,
        exconds: ExistenceCondition,
    },
    Dirt(BlockDescriptor),
    Grass(BlockDescriptor),
    Stone(BlockDescriptor),
    Greenery(BlockDescriptor),
    Sand(BlockDescriptor),
}
