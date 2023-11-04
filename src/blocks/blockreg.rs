use super::{xsprite_mesh::generate_xsprite_mesh, *};
use crate::prelude::*;
#[derive(Resource, Clone)]
pub struct BlockRegistry {
    dirt_mesh: Mesh,
    grass_mesh: Mesh,
    stone_mesh: Mesh,
    greenery_mesh: Mesh,
}

impl VoxelRegistry for BlockRegistry {
    type Voxel = Block;
    fn get_mesh(&self, voxel: &Self::Voxel) -> VoxelMesh<&Mesh> {
        match voxel {
            Block::AIR => VoxelMesh::Null,
            Block::GRASS => VoxelMesh::NormalCube(&self.grass_mesh),
            Block::STONE => VoxelMesh::NormalCube(&self.stone_mesh),
            Block::DIRT => VoxelMesh::NormalCube(&self.dirt_mesh),
            Block::GREENERY => VoxelMesh::CustomMesh(&self.greenery_mesh),
        }
    }

    fn all_attributes(&self) -> Vec<MeshVertexAttribute> {
        vec![
            Mesh::ATTRIBUTE_POSITION,
            Mesh::ATTRIBUTE_UV_0,
            Mesh::ATTRIBUTE_COLOR,
            Mesh::ATTRIBUTE_NORMAL,
        ]
    }

    fn get_voxel_dimensions(&self) -> [f32; 3] {
        VOXEL_DIMS
    }

    fn get_center(&self) -> [f32; 3] {
        VOXEL_CENTER
    }

    fn is_covering(&self, voxel: &Self::Voxel, _side: prelude::Face) -> bool {
        *voxel != Block::AIR && *voxel != Block::GREENERY
    }
}

impl Default for BlockRegistry {
    fn default() -> Self {
        BlockRegistry {
            dirt_mesh: generate_voxel_mesh(
                VOXEL_DIMS,
                TEXTURE_ATLAS_DIMS,
                [
                    (Top, [2, 0]),
                    (Bottom, [2, 0]),
                    (Right, [2, 0]),
                    (Left, [2, 0]),
                    (Back, [2, 0]),
                    (Forward, [2, 0]),
                ],
                VOXEL_CENTER,
                PADDING,
                Some(COLOR_INTENSITY),
                ALPHA,
            ),
            grass_mesh: generate_voxel_mesh(
                VOXEL_DIMS,
                TEXTURE_ATLAS_DIMS,
                [
                    (Top, [0, 0]),
                    (Bottom, [2, 0]),
                    (Right, [1, 0]),
                    (Left, [1, 0]),
                    (Back, [1, 0]),
                    (Forward, [1, 0]),
                ],
                VOXEL_CENTER,
                PADDING,
                Some(COLOR_INTENSITY),
                ALPHA,
            ),
            stone_mesh: generate_voxel_mesh(
                VOXEL_DIMS,
                TEXTURE_ATLAS_DIMS,
                [
                    (Top, [3, 0]),
                    (Bottom, [3, 0]),
                    (Right, [3, 0]),
                    (Left, [3, 0]),
                    (Back, [3, 0]),
                    (Forward, [3, 0]),
                ],
                VOXEL_CENTER,
                PADDING,
                Some(COLOR_INTENSITY),
                ALPHA,
            ),
            greenery_mesh: generate_xsprite_mesh(
                VOXEL_DIMS,
                TEXTURE_ATLAS_DIMS,
                [4, 0],
                VOXEL_CENTER,
                PADDING,
                Some(COLOR_INTENSITY),
                ALPHA,
                GREENERY_SCALE,
            ),
        }
    }
}
