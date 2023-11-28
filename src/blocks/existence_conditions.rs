use super::*;

pub type BlockCondition = Box<dyn Fn(Block) -> bool + Send + Sync + 'static>;

pub trait CommonBlockCondition {
    fn id_equals(id: BlockId) -> BlockCondition;
    fn equals(block: Block) -> BlockCondition;
}

impl CommonBlockCondition for BlockCondition {
    fn id_equals(id: BlockId) -> BlockCondition {
        Box::new(move |block| block as BlockId == id)
    }

    fn equals(block: Block) -> BlockCondition {
        Box::new(move |b| b == block)
    }
}

pub enum ExistenceCondition {
    Always,
    Never,
    BlockUnderMust(BlockCondition),
    BlockToTheSideMust(Face, BlockCondition),
    AND(Vec<ExistenceCondition>),
    OR(Vec<ExistenceCondition>),
}
