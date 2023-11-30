use super::properties::{
    BlockProperty, DynamicProperty, PassiveProperty, PerceptibleProperty, PhysicalProperty,
};
use super::*;
use novacraft_meshing_backend::prelude::Face;

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

#[derive(Default)]
pub struct BlockDescriptor {
    pub mesh_gen_data: MeshGenData,
    pub physical: PropertyCollection<PhysicalProperty>,
    pub passive: PropertyCollection<PassiveProperty>,
    pub perceptible: PropertyCollection<PerceptibleProperty>,
    pub dynamic: PropertyCollection<DynamicProperty>,
}

#[derive(Default)]
pub enum MeshGenData {
    Cube(CubeTextureCords),
    XSprite(XSpriteTextureCords),
    External(ExternalMesh<CubeTextureCords>),
    #[default]
    Air,
}

impl MeshGenData {
    pub fn get_path(&self) -> &str {
        match self {
            Self::External(ext) => ext.mesh_asset.get_path(),
            _ => panic!("Can only get path from Externally imported meshes"),
        }
    }
}

impl Into<Mesh> for MeshGenData {
    fn into(self) -> Mesh {
        match self {
            Self::Air => Mesh::new(PrimitiveTopology::TriangleList),
            Self::Cube(t) => t.into(),
            Self::XSprite(t) => t.into(),
            Self::External(t) => t.into(),
        }
    }
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
    pub sprite: AtlasDims,
}

impl Into<Mesh> for CubeTextureCords {
    fn into(self) -> Mesh {
        generate_voxel_mesh(
            VOXEL_DIMS,
            TEXTURE_ATLAS_DIMS,
            [
                (Top, self.top),
                (Bottom, self.bottom),
                (Right, self.right),
                (Left, self.left),
                (Back, self.back),
                (Forward, self.forward),
            ],
            VOXEL_CENTER,
            PADDING,
            Some(COLOR_INTENSITY),
            ALPHA,
        )
    }
}

impl Into<Mesh> for XSpriteTextureCords {
    fn into(self) -> Mesh {
        generate_xsprite_mesh(
            VOXEL_DIMS,
            TEXTURE_ATLAS_DIMS,
            self.sprite,
            VOXEL_CENTER,
            PADDING,
            Some(COLOR_INTENSITY),
            ALPHA,
            XSPRITE_SCALE,
        )
    }
}

impl Into<Mesh> for ExternalMesh<CubeTextureCords> {
    fn into(self) -> Mesh {
        return self.while_loading.into();
    }
}

impl XSpriteTextureCords {
    pub fn uniform(dims: AtlasDims) -> Self {
        Self { sprite: dims }
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

pub type AtlasDims = [u32; 2];

pub enum CustomMeshAsset {
    PathToMesh(&'static str),
}

impl CustomMeshAsset {
    pub fn from_path(path: &'static str) -> CustomMeshAsset {
        CustomMeshAsset::PathToMesh(path)
    }

    pub fn get_path(&self) -> &str {
        match self {
            Self::PathToMesh(path) => *path,
        }
    }
}

pub struct ExternalMesh<M: Into<Mesh>> {
    mesh_asset: CustomMeshAsset,
    while_loading: M,
}
