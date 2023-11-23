use super::*;

pub type ExistenceCondition = Box<dyn Fn(Block) -> bool + Send + Sync + 'static>;

pub enum ConditionalExistence {
    Always,
    Never,
    BlockUnderMust(ExistenceCondition),
    BlockToTheSideMust(Face, ExistenceCondition),
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
            sand: ConditionalExistence::Never,
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
