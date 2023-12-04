// REFACTORED

use super::*;

/// Simple condition. Takes in a block as input and returns `true` or `false`. For example, it can
/// be defined to only return true if the block is a specific block, or a variety of blocks.
pub type BlockCondition = Box<dyn Fn(Block) -> bool + Send + Sync + 'static>;

pub trait CommonBlockCondition {
    fn equals(block: Block) -> BlockCondition;
}

impl CommonBlockCondition for BlockCondition {
    /// Define a condition that returns `true` only if a future parameter is equal the given `block`.
    fn equals(block: Block) -> BlockCondition {
        Box::new(move |b| b == block)
    }
}

/// Defining specific logic using [`BlockCondition`(s)](BlockCondition).
pub enum ExistenceCondition {
    /// Always true
    Always,
    /// Always false
    Never,
    /// The block to the side must pass a condition.
    BlockToTheSideMust(Face, BlockCondition),
    /// Chaining multiple [`ExistenceCondition`] together, return true if all true.
    ALL(Vec<ExistenceCondition>),
    /// Chaining multiple [`ExistenceCondition`] together, return true if any is true.
    ANY(Vec<ExistenceCondition>),
}

/// The data used to evaluate (solve) the [`ExistenceCondition`]
/// By default, not all data is required, but if the solver will ask for data that will not be
/// present in the [`ExistenceConditionSolverData`] (set to [`None`]) the solver will panic.
#[derive(Clone, Copy)]
pub struct ExistenceConditionSolverData {
    pub surrounding_blocks: [Option<Block>; 6],
}

impl ExistenceCondition {
    /// Evaluate self to be true or false given [`solver_data`](ExistenceConditionSolverData)
    pub fn solve(&self, solver_data: ExistenceConditionSolverData) -> bool {
        match self {
            ExistenceCondition::Always => true,
            ExistenceCondition::Never => false,
            ExistenceCondition::BlockToTheSideMust(face, cond) => {
                cond(solver_data.surrounding_blocks[*face as usize].expect("Incomplete data given to ExistenceCondition solver, expected Some(block) found None"))
            }
            ExistenceCondition::ALL(exconds) => {
                let mut finale = true;
                for excond in exconds {
                    finale = finale && excond.solve(solver_data);
                }
                finale
            }
            ExistenceCondition::ANY(exconds) => {
                let mut finale = false;
                for excond in exconds {
                    finale = finale || excond.solve(solver_data);
                }
                finale
            }
        }
    }
}
