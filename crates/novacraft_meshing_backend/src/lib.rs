pub(crate) mod adj;
pub(crate) mod face;
pub(crate) mod mesh_metadata;
pub mod mesh_utils;
pub(crate) mod meshem;
pub(crate) mod pbs;
pub(crate) mod update;
pub mod util;
pub(crate) mod voxel_mesh;

use bevy::prelude::*;
use bevy::render::mesh::MeshVertexAttribute;

pub mod prelude {
    pub use crate::adj::*;
    pub use crate::face::Face::*;
    pub use crate::face::*;
    pub use crate::mesh_metadata::*;
    pub use crate::meshem::*;
    pub use crate::pbs::*;
    pub use crate::update::*;
    pub(crate) use crate::util::compressed_voxel_grid::*;
    pub use crate::util::vav::*;
    pub use crate::util::*;
    pub use crate::voxel_mesh::*;
    pub use crate::VoxelRegistry;
    pub use crate::*;
    pub use mesh_utils::xsprite_mesh::*;
}

/// Implementing this trait for your own data-structure is the most important
/// prerequesite if you want to use the crate.
pub trait VoxelRegistry {
    type Voxel: std::fmt::Debug + Eq + PartialEq + Sized + Clone + Copy;
    /// Returns None if the mesh is "irrelevant" as in it's air or not a Voxel.
    fn get_mesh(&self, voxel: &Self::Voxel) -> VoxelMesh<&Mesh>;
    /// Would this voxel cover the voxel that's located on it's `side`? for example, an air block
    /// would not cover any side, but a slab would only cover the bottom.
    fn is_covering(&self, voxel: &Self::Voxel, side: prelude::Face) -> bool;
    /// The center of the voxel (physical center, the center of the default block is 0,0,0 eg)
    fn get_center(&self) -> [f32; 3];
    /// All the voxels must have standard and equal dimesions (y is up).
    fn get_voxel_dimensions(&self) -> [f32; 3];
    /// The attributes we are considering while meshing the grid.
    fn all_attributes(&self) -> Vec<MeshVertexAttribute>;
}

/// (width, height, length) - note that bevy considers the "y position" to be height.
pub type Dimensions = (usize, usize, usize);

#[derive(Clone)]
pub enum VoxelMesh<T> {
    NormalCube(T),
    XSprite(T),
    CustomMesh(T),
    Null,
}

impl<T> VoxelMesh<T> {
    pub fn unwrap(self) -> T {
        match self {
            Self::NormalCube(t) => t,
            Self::XSprite(t) => t,
            Self::CustomMesh(t) => t,
            Self::Null => panic!("Triead unwrapping a Null VoxelMesh type."),
        }
    }

    pub fn expect(self, msg: &str) -> T {
        match self {
            Self::NormalCube(t) => t,
            Self::XSprite(t) => t,
            Self::CustomMesh(t) => t,
            Self::Null => panic!("{}", msg),
        }
    }

    pub fn ref_mesh(&self) -> VoxelMesh<&T> {
        match self {
            VoxelMesh::NormalCube(t) => VoxelMesh::NormalCube(t),
            VoxelMesh::XSprite(t) => VoxelMesh::XSprite(t),
            VoxelMesh::CustomMesh(t) => VoxelMesh::CustomMesh(t),
            VoxelMesh::Null => VoxelMesh::Null,
        }
    }

    pub fn set(&mut self, new_mesh: T) {
        match self {
            VoxelMesh::NormalCube(t) => *t = new_mesh,
            VoxelMesh::XSprite(t) => *t = new_mesh,
            VoxelMesh::CustomMesh(t) => *t = new_mesh,
            VoxelMesh::Null => {}
        }
    }
}

/// [+y, -y, +x, -x, +z, -z], true if that face is not covered.
pub(crate) type Neighbors = [bool; 6];
