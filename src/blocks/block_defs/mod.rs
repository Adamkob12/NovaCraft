#![allow(dead_code)]
use super::{
    block_descriptor::*,
    dynamic_property::{BlockTransformation, CommonBlockTransformations},
    existence_conditions::ExistenceCondition,
    properties::{DynamicProperty, PassiveProperty, PhysicalProperty},
    Block, Face,
};

#[allow(non_snake_case)]
impl BlockDescriptor {
    pub fn Air() -> Self {
        Self {
            passive: PropertyCollection::<PassiveProperty>::from_property(
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
            dynamic: PropertyCollection::<DynamicProperty>::from_property(
                DynamicProperty::BlockTransformIf(
                    ExistenceCondition::BlockToTheSideMust(
                        Face::Top,
                        Box::new({
                            |block| match block {
                                Block::AIR | Block::GREENERY => false,
                                _ => true,
                            }
                        }),
                    ),
                    BlockTransformation::transform_into(Block::DIRT),
                ),
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
            passive: PropertyCollection::<PassiveProperty>::from_property(
                PassiveProperty::YieldToFallingBlock,
            ),
            dynamic: PropertyCollection::<DynamicProperty>::from_property(
                DynamicProperty::ExistenceCondition(ExistenceCondition::BlockToTheSideMust(
                    Face::Bottom,
                    Box::new(|block| block == Block::GRASS),
                )),
            ),
            ..Default::default()
        }
    }

    pub fn Sand() -> Self {
        BlockDescriptor {
            mesh_gen_data: MeshGenData::Cube(CubeTextureCords::uniform([6, 0])),
            physical: PropertyCollection::<PhysicalProperty>::from_property(
                PhysicalProperty::AffectedByGravity,
            ),
            ..Default::default()
        }
    }
}
