use super::*;

pub type BlockTransformation = Box<dyn Fn(Block) -> Block + Send + Sync + 'static>;

pub trait CommonBlockTransformations {
    fn transform_into(block: Block) -> BlockTransformation;
}

impl CommonBlockTransformations for BlockTransformation {
    fn transform_into(block: Block) -> BlockTransformation {
        Box::new(move |_| block)
    }
}
