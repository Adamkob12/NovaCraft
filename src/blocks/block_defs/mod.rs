#![allow(dead_code)]
use super::{
    block_descriptor::*,
    existence_conditions::{BlockCondition, CommonBlockCondition, ConditionalExistence},
    properties::{PassiveProperty, PhysicalProperty},
    Face,
};

#[allow(non_snake_case)]
impl BlockDescriptor {
    pub fn Air() -> Self {
        Self {
            passive_properties: PropertyCollection::<PassiveProperty>::from_property(
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
            existence_conditions: ConditionalExistence::BlockUnderMust(BlockCondition::id_equals(
                1,
            )),
            passive_properties: PropertyCollection::<PassiveProperty>::from_property(
                PassiveProperty::YieldToFallingBlock,
            ),
            ..Default::default()
        }
    }

    pub fn Sand() -> Self {
        BlockDescriptor {
            mesh_gen_data: MeshGenData::Cube(CubeTextureCords::uniform([6, 0])),
            physical_properties: PropertyCollection::<PhysicalProperty>::from_property(
                PhysicalProperty::AffectedByGravity,
            ),
            ..Default::default()
        }
    }
}
