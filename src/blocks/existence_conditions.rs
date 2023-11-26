use super::*;

pub type BlockCondition = Box<dyn Fn(Block) -> bool + Send + Sync + 'static>;

pub trait CommonBlockCondition {
    fn id_equals(id: BlockId) -> BlockCondition;
}

impl CommonBlockCondition for BlockCondition {
    fn id_equals(id: BlockId) -> BlockCondition {
        Box::new(move |block| block as BlockId == id)
    }
}

pub enum ConditionalExistence {
    Always,
    Never,
    BlockUnderMust(BlockCondition),
    BlockToTheSideMust(Face, BlockCondition),
    AND(Vec<ConditionalExistence>),
    OR(Vec<ConditionalExistence>),
}

#[derive(Resource)]
pub struct ExistenceConditions {
    air: ConditionalExistence,
    dirt: ConditionalExistence,
    grass: ConditionalExistence,
    stone: ConditionalExistence,
    greenery: ConditionalExistence,
    sand: ConditionalExistence,
}

impl Default for ExistenceConditions {
    fn default() -> Self {
        Self {
            air: ConditionalExistence::Always,
            dirt: ConditionalExistence::Always,
            grass: ConditionalExistence::Always,
            stone: ConditionalExistence::Always,
            greenery: ConditionalExistence::BlockUnderMust(Box::new(|block: Block| {
                block == Block::GRASS
            })),
            sand: ConditionalExistence::Always,
        }
    }
}

impl ExistenceConditions {
    pub fn get_condition(&self, block: &Block) -> &ConditionalExistence {
        match block {
            Block::AIR => &self.air,
            Block::DIRT => &self.dirt,
            Block::GRASS => &self.grass,
            Block::STONE => &self.stone,
            Block::GREENERY => &self.greenery,
            Block::SAND => &self.sand,
        }
    }
}
