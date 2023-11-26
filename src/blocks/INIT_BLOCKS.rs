use super::block_descriptor::*;
use super::properties::*;
use novacraft_derive::InitBlocks;

#[allow(non_snake_case, non_camel_case_types)]
#[derive(InitBlocks)]
pub enum BLOCKS_INITIALIZER {
    Air(BlockDescriptor),
    Dirt(BlockDescriptor),
    Grass(BlockDescriptor),
    Stone(BlockDescriptor),
    Greenery(BlockDescriptor),
    Sand(BlockDescriptor),
}
