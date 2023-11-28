use super::*;

pub type BlockTransformation = Box<dyn Fn(Block) -> Block + Send + Sync + 'static>;
