use super::*;
const BLOCK_DENSITY: f32 = 100.0;

#[derive(Component)]
pub struct FallingBlock;

#[derive(Clone, Copy, PartialEq)]
pub enum BlockProperty {
    AffectedByGravity,
    YieldToFallingBlock,
    MustBeOnTopOf(Block),
}

#[derive(Resource)]
pub struct BlockPropertyRegistry {
    air: Vec<BlockProperty>,
    dirt: Vec<BlockProperty>,
    grass: Vec<BlockProperty>,
    stone: Vec<BlockProperty>,
    greenery: Vec<BlockProperty>,
    sand: Vec<BlockProperty>,
}

impl Default for BlockPropertyRegistry {
    fn default() -> Self {
        BlockPropertyRegistry {
            air: vec![BlockProperty::YieldToFallingBlock],
            dirt: vec![],
            grass: vec![],
            stone: vec![],
            greenery: vec![
                BlockProperty::YieldToFallingBlock,
                BlockProperty::MustBeOnTopOf(Block::GRASS),
            ],
            sand: vec![BlockProperty::AffectedByGravity],
        }
    }
}

impl BlockPropertyRegistry {
    pub fn iter_properties(&self, block: &Block) -> &[BlockProperty] {
        match block {
            Block::AIR => self.air.as_slice(),
            Block::DIRT => self.dirt.as_slice(),
            Block::GRASS => self.grass.as_slice(),
            Block::STONE => self.stone.as_slice(),
            Block::GREENERY => self.greenery.as_slice(),
            Block::SAND => self.sand.as_slice(),
        }
    }

    pub fn contains_property(&self, block: &Block, property: &BlockProperty) -> bool {
        match block {
            Block::AIR => self.air.contains(property),
            Block::DIRT => self.dirt.contains(property),
            Block::GRASS => self.grass.contains(property),
            Block::STONE => self.stone.contains(property),
            Block::GREENERY => self.greenery.contains(property),
            Block::SAND => self.sand.contains(property),
        }
    }

    pub fn get_density(&self, block: &Block) -> f32 {
        match block {
            Block::AIR => 0.0,
            _ => BLOCK_DENSITY,
        }
    }

    pub fn is_collidable(block: &Block) -> bool {
        match block {
            Block::AIR => false,
            Block::DIRT => true,
            Block::GRASS => true,
            Block::STONE => true,
            Block::GREENERY => false,
            Block::SAND => true,
        }
    }
}
