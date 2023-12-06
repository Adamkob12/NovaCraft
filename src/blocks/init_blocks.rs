// REFACTORED
use super::block_descriptor::*;
use super::properties::*;
use crate::AssetLoadingState;
pub use novacraft_derive::InitBlocks;

/// Here we decalre all of the blocks we want to use in the game. Their name must be the same as
/// the name of their description method, and the value in the paranthesis is the path to their
/// description method.
/// When declaring Air (all games have air in them) also specify all of the Properties.
/// This generates everything the game needs to run, specifically the `BlockRegistiesPlugin`
/// The syntax and semantics will change, with the planned derive macro revamp.
#[allow(non_snake_case, non_camel_case_types)]
#[derive(InitBlocks)]
pub enum __InitBlocks__ {
    Air {
        block_descriptor: BlockDescriptor,
        physical: PhysicalProperty,
        passive: PassiveProperty,
        perceptible: PerceptibleProperty,
        dynamic: DynamicProperty,
    },
    Dirt(BlockDescriptor),
    Grass(BlockDescriptor),
    Stone(BlockDescriptor),
    Greenery(BlockDescriptor),
    Sand(BlockDescriptor),
}
