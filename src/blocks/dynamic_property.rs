/// REFACTORED
use super::*;

/// `BlockTransformation` is a simple function that transforms a block into a different block.
/// for example, you can define it to transform everything into a stone block. Or only transform
/// a grass block into a dirt block.
pub type BlockTransformation = Box<dyn Fn(Block) -> Block + Send + Sync + 'static>;

pub trait CommonBlockTransformations {
    fn transform_into(block: Block) -> BlockTransformation;
}

impl CommonBlockTransformations for BlockTransformation {
    /// No matter what the block is, transform it into a given block.
    fn transform_into(block: Block) -> BlockTransformation {
        Box::new(move |_| block)
    }
}
