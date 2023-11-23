#![allow(dead_code)]
use novacraft_meshing_backend::prelude::Face;

use super::{
    existence_conditions::ConditionalExistence,
    properties::{PassiveProperty, PerceptibleProperty, PhysicalProperty},
};

pub struct PhysicalProperties(pub Vec<PhysicalProperty>);
pub struct PassiveProperties(pub Vec<PassiveProperty>);
pub struct PerceptibleProperties(pub Vec<PerceptibleProperty>);

impl std::ops::Deref for PhysicalProperties {
    type Target = Vec<PhysicalProperty>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Deref for PassiveProperties {
    type Target = Vec<PassiveProperty>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Deref for PerceptibleProperties {
    type Target = Vec<PerceptibleProperty>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct BlockDescriptor {
    name: String,
    mesh_gen_data: MeshGenData,
    existence_conditions: ConditionalExistence,
    physical_properties: PhysicalProperties,
    passive_properties: PassiveProperties,
    perspectible_properties: PerceptibleProperties,
}

pub enum MeshGenData {
    Cube(CubeTextureCords),
    XSprite(XSpriteTextureCords),
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
