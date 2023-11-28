#![allow(dead_code)]

use novacraft_meshing_backend::prelude::Face;

use super::{
    existence_conditions::ExistenceCondition,
    properties::{
        BlockProperty, BlockPropertyTypes, DynamicProperty, PassiveProperty, PerceptibleProperty,
        PhysicalProperty,
    },
};

pub struct PropertyCollection<T: BlockProperty>(pub Vec<T>);

impl<T: BlockProperty> PropertyCollection<T> {
    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn with_property(mut self, prop: T) -> Self {
        self.0.push(prop);
        self
    }

    pub fn from_property(prop: T) -> Self {
        Self(vec![prop])
    }
}

impl<T: BlockProperty> Default for PropertyCollection<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: BlockProperty> std::ops::Deref for PropertyCollection<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(non_camel_case_types)]
pub struct INIT_BLOCK_DESCRIPTOR;

#[allow(non_snake_case)]
#[derive(Default)]
pub struct BlockDescriptor {
    pub mesh_gen_data: MeshGenData,
    pub ExistenceConditions: PropertyCollection<ExistenceCondition>,
    pub PhysicalPropertys: PropertyCollection<PhysicalProperty>,
    pub PassivePropertys: PropertyCollection<PassiveProperty>,
    pub PerceptiblePropertys: PropertyCollection<PerceptibleProperty>,
    pub DynamicPropertys: PropertyCollection<DynamicProperty>,
}

#[derive(Default)]
pub enum MeshGenData {
    Cube(CubeTextureCords),
    XSprite(XSpriteTextureCords),
    #[default]
    Air,
}

pub struct CubeTextureCords {
    pub top: AtlasDims,
    pub bottom: AtlasDims,
    pub right: AtlasDims,
    pub left: AtlasDims,
    pub back: AtlasDims,
    pub forward: AtlasDims,
}

pub struct XSpriteTextureCords {
    pub sprite1: AtlasDims,
    pub sprite2: AtlasDims,
}

impl XSpriteTextureCords {
    pub fn uniform(dims: AtlasDims) -> Self {
        Self {
            sprite1: dims,
            sprite2: dims,
        }
    }
}

impl CubeTextureCords {
    pub fn uniform(dims: AtlasDims) -> Self {
        Self {
            top: dims,
            bottom: dims,
            right: dims,
            left: dims,
            back: dims,
            forward: dims,
        }
    }

    pub fn with_face(mut self, face: Face, dims: AtlasDims) -> Self {
        match face {
            Face::Top => self.top = dims,
            Face::Bottom => self.bottom = dims,
            Face::Right => self.right = dims,
            Face::Left => self.left = dims,
            Face::Back => self.back = dims,
            Face::Forward => self.forward = dims,
        }
        self
    }
}

pub type AtlasDims = [usize; 2];
