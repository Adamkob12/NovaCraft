use super::{block_descriptor::PropertyCollection, *};
const BLOCK_DENSITY: f32 = 100.0;

pub trait Property {}

#[derive(Component)]
pub struct FallingBlock;

#[derive(Clone, Copy, PartialEq)]
pub enum PhysicalProperty {
    AffectedByGravity,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PassiveProperty {
    YieldToFallingBlock,
}

#[derive(Clone, Copy)]
pub enum PerceptibleProperty {
    LightSource(PointLight),
    // TODO: audio stuff
    AudioSource,
}

impl Property for PerceptibleProperty {}
impl Property for PassiveProperty {}
impl Property for PhysicalProperty {}

#[derive(Resource)]
pub struct BlockPropertyRegistry<P: Property> {
    pub air: PropertyCollection<P>,
    pub dirt: PropertyCollection<P>,
    pub grass: PropertyCollection<P>,
    pub stone: PropertyCollection<P>,
    pub greenery: PropertyCollection<P>,
    pub sand: PropertyCollection<P>,
}

impl<P: Property + PartialEq> BlockPropertyRegistry<P> {
    pub fn iter_properties(&self, block: &Block) -> &[P] {
        match block {
            Block::AIR => self.air.0.as_slice(),
            Block::DIRT => self.dirt.0.as_slice(),
            Block::GRASS => self.grass.0.as_slice(),
            Block::STONE => self.stone.0.as_slice(),
            Block::GREENERY => self.greenery.0.as_slice(),
            Block::SAND => self.sand.0.as_slice(),
        }
    }

    pub fn contains_property(&self, block: &Block, property: &P) -> bool {
        match block {
            Block::AIR => self.air.contains(property),
            Block::DIRT => self.dirt.contains(property),
            Block::GRASS => self.grass.contains(property),
            Block::STONE => self.stone.contains(property),
            Block::GREENERY => self.greenery.contains(property),
            Block::SAND => self.sand.contains(property),
        }
    }
}

impl BlockPropertyRegistry<PhysicalProperty> {
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
