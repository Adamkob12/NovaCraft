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
    BlockToTheSideMust(Face, BlockCondition),
    AND(Vec<ExistenceCondition>),
    OR(Vec<ExistenceCondition>),
}

#[derive(Clone, Copy)]
pub struct ExistenceConditionSolverData {
    pub surrounding_blocks: [Option<Block>; 6],
}

impl ExistenceCondition {
    pub fn solve(&self, solver_data: ExistenceConditionSolverData) -> bool {
        match self {
            ExistenceCondition::Always => true,
            ExistenceCondition::Never => false,
            ExistenceCondition::BlockToTheSideMust(face, cond) => {
                cond(solver_data.surrounding_blocks[*face as usize].expect("Incomplete data given to ExistenceCondition solver, expected Some(block) found None"))
            }
            ExistenceCondition::AND(exconds) => {
                let mut finale = true;
                for excond in exconds {
                    finale = finale && excond.solve(solver_data);
                }
                finale
            }
            ExistenceCondition::OR(exconds) => {
                let mut finale = false;
                for excond in exconds {
                    finale = finale || excond.solve(solver_data);
                }
                finale
            }
        }
    }
}
