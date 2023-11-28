#![allow(dead_code)]
use super::{
    block_descriptor::*,
    existence_conditions::{BlockCondition, CommonBlockCondition, ExistenceCondition},
    properties::{DynamicProperty, PassiveProperty, PhysicalProperty},
    Block, Face,
};

#[allow(non_snake_case)]
impl BlockDescriptor {
    pub fn Air() -> Self {
        Self {
            PassivePropertys: PropertyCollection::<PassiveProperty>::from_property(
                PassiveProperty::YieldToFallingBlock,
            ),
            ..Default::default()
        }
    }

    pub fn Grass() -> Self {
        BlockDescriptor {
            mesh_gen_data: MeshGenData::Cube(
                CubeTextureCords::uniform([1, 0])
                    .with_face(Face::Top, [0, 0])
                    .with_face(Face::Bottom, [2, 0]),
            ),
            DynamicPropertys: PropertyCollection::<DynamicProperty>::from_property(
                DynamicProperty::BlockAbove(Box::new({
                    |block| match block {
                        Block::AIR | Block::GREENERY => Block::GRASS,
                        _ => Block::DIRT,
                    }
                })),
            ),
            ..Default::default()
        }
    }

    pub fn Stone() -> Self {
        BlockDescriptor {
            mesh_gen_data: MeshGenData::Cube(CubeTextureCords::uniform([3, 0])),
            ..Default::default()
        }
    }

    pub fn Dirt() -> Self {
        BlockDescriptor {
            mesh_gen_data: MeshGenData::Cube(CubeTextureCords::uniform([2, 0])),
            ..Default::default()
        }
    }

    pub fn Greenery() -> Self {
        BlockDescriptor {
            mesh_gen_data: MeshGenData::XSprite(XSpriteTextureCords::uniform([4, 0])),
            ExistenceConditions: PropertyCollection::<ExistenceCondition>::from_property(
                ExistenceCondition::BlockUnderMust(BlockCondition::equals(Block::GRASS)),
            ),
            ..Default::default()
        }
    }

    pub fn Sand() -> Self {
        BlockDescriptor {
            mesh_gen_data: MeshGenData::Cube(CubeTextureCords::uniform([6, 0])),
            PhysicalPropertys: PropertyCollection::<PhysicalProperty>::from_property(
                PhysicalProperty::AffectedByGravity,
            ),
            ..Default::default()
        }
    }
}
