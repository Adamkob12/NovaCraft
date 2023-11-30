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
    pub mesh_builder: MeshBuilder,
    pub physical: PropertyCollection<PhysicalProperty>,
    pub passive: PropertyCollection<PassiveProperty>,
    pub perceptible: PropertyCollection<PerceptibleProperty>,
    pub dynamic: PropertyCollection<DynamicProperty>,
}

#[derive(Default)]
pub enum MeshBuilder {
    Cube(CubeMeshBuilder),
    XSprite(XSpriteMeshBuilder),
    External(ExternalMesh<CubeMeshBuilder>),
    #[default]
    Null,
}

impl Into<VoxelMesh<Mesh>> for MeshBuilder {
    fn into(self) -> VoxelMesh<Mesh> {
        match self {
            Self::Null => VoxelMesh::Null,
            Self::Cube(t) => VoxelMesh::NormalCube(t.into()),
            Self::XSprite(t) => VoxelMesh::XSprite(t.into()),
            Self::External(t) => VoxelMesh::CustomMesh(t.into()),
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

impl Into<CubeMeshBuilder> for CubeTextureCords {
    fn into(self) -> CubeMeshBuilder {
        CubeMeshBuilder::from_cube_texture_cords(self)
    }
}

impl Into<XSpriteMeshBuilder> for XSpriteTextureCords {
    fn into(self) -> XSpriteMeshBuilder {
        XSpriteMeshBuilder::from_xsprite_texture_cords(self)
    }
}

impl Into<Mesh> for ExternalMesh<CubeMeshBuilder> {
    fn into(self) -> Mesh {
        return self.alt_mesh.into();
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

pub struct ExternalMesh<M: Into<Mesh>> {
    alt_mesh: M,
}

pub struct CubeMeshBuilder {
    voxel_dims: [f32; 3],
    voxel_center: [f32; 3],
    texture_atlas_dims: [u32; 2],
    padding: f32,
    color_intensity: f32,
    alpha: f32,
    cube_texture_cords: CubeTextureCords,
}

#[allow(dead_code)]
impl CubeMeshBuilder {
    pub fn from_cube_texture_cords(cube_texture_cords: CubeTextureCords) -> CubeMeshBuilder {
        Self {
            voxel_dims: VOXEL_DIMS,
            voxel_center: VOXEL_CENTER,
            texture_atlas_dims: TEXTURE_ATLAS_DIMS,
            padding: PADDING,
            color_intensity: COLOR_INTENSITY,
            alpha: ALPHA,
            cube_texture_cords,
        }
    }

    pub fn build(self) -> Mesh {
        generate_voxel_mesh(
            self.voxel_dims,
            self.texture_atlas_dims,
            [
                (Top, self.cube_texture_cords.top),
                (Bottom, self.cube_texture_cords.bottom),
                (Right, self.cube_texture_cords.right),
                (Left, self.cube_texture_cords.left),
                (Back, self.cube_texture_cords.back),
                (Forward, self.cube_texture_cords.forward),
            ],
            self.voxel_center,
            self.padding,
            Some(self.color_intensity),
            self.alpha,
        )
    }

    pub fn override_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }
    pub fn override_color_intensity(mut self, color_intensity: f32) -> Self {
        self.color_intensity = color_intensity;
        self
    }
    pub fn override_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
    pub fn override_texture_atlas_dims(mut self, texture_atlas_dims: [u32; 2]) -> Self {
        self.texture_atlas_dims = texture_atlas_dims;
        self
    }
    pub fn override_voxel_center(mut self, voxel_center: [f32; 3]) -> Self {
        self.voxel_center = voxel_center;
        self
    }
    pub fn override_voxel_dims(mut self, voxel_dims: [f32; 3]) -> Self {
        self.voxel_dims = voxel_dims;
        self
    }
}

pub struct XSpriteMeshBuilder {
    voxel_dims: [f32; 3],
    voxel_center: [f32; 3],
    texture_atlas_dims: [u32; 2],
    padding: f32,
    color_intensity: f32,
    alpha: f32,
    xsprite_scale: f32,
    xsprite_texture_cords: XSpriteTextureCords,
}

#[allow(dead_code)]
impl XSpriteMeshBuilder {
    pub fn from_xsprite_texture_cords(
        xsprite_texture_cords: XSpriteTextureCords,
    ) -> XSpriteMeshBuilder {
        Self {
            voxel_dims: VOXEL_DIMS,
            voxel_center: VOXEL_CENTER,
            texture_atlas_dims: TEXTURE_ATLAS_DIMS,
            padding: PADDING,
            color_intensity: COLOR_INTENSITY,
            alpha: ALPHA,
            xsprite_scale: XSPRITE_SCALE,
            xsprite_texture_cords,
        }
    }

    pub fn build(self) -> Mesh {
        generate_xsprite_mesh(
            self.voxel_dims,
            self.texture_atlas_dims,
            self.xsprite_texture_cords.sprite,
            self.voxel_center,
            self.padding,
            Some(self.color_intensity),
            self.alpha,
            self.xsprite_scale,
        )
    }

    pub fn override_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }
    pub fn override_color_intensity(mut self, color_intensity: f32) -> Self {
        self.color_intensity = color_intensity;
        self
    }
    pub fn override_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
    pub fn override_texture_atlas_dims(mut self, texture_atlas_dims: [u32; 2]) -> Self {
        self.texture_atlas_dims = texture_atlas_dims;
        self
    }
    pub fn override_voxel_center(mut self, voxel_center: [f32; 3]) -> Self {
        self.voxel_center = voxel_center;
        self
    }
    pub fn override_voxel_dims(mut self, voxel_dims: [f32; 3]) -> Self {
        self.voxel_dims = voxel_dims;
        self
    }
    pub fn override_xsprite_scale(mut self, xsprite_scale: f32) -> Self {
        self.xsprite_scale = xsprite_scale;
        self
    }
}

impl Into<Mesh> for CubeMeshBuilder {
    fn into(self) -> Mesh {
        self.build()
    }
}

impl Into<Mesh> for XSpriteMeshBuilder {
    fn into(self) -> Mesh {
        self.build()
    }
}
