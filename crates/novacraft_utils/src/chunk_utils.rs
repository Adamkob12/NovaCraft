use bevy_math::prelude::*;
use novacraft_meshing_backend::prelude::Face;

pub type Dimensions = UVec3;
pub type ChunkCords = IVec2;
pub struct ChunkGrid<T: Copy> {
    dims: Dimensions,
    grid: [T],
}
pub type BlockPos = UVec3;
pub type BlockIndex = usize;

impl<T: Copy> ChunkGrid<T> {
    pub fn get_block(&self, block_pos: BlockPos) -> Option<T> {
        pos_to_index(block_pos, self.dims).map(|i| self.grid[i])
    }

    pub fn get_block_or(&self, block_pos: BlockPos, default: T) -> T {
        pos_to_index(block_pos, self.dims).map_or(default, |i| self.grid[i])
    }

    pub fn get_neighbor_of(&self, block_pos: BlockPos, face: Face) -> Option<T> {
        neighbor_of(block_pos, face, self.dims).map(|i| self.grid[i])
    }

    pub fn get_neighbor_of_or(&self, block_pos: BlockPos, face: Face, default: T) -> T {
        neighbor_of(block_pos, face, self.dims).map_or(default, |i| self.grid[i])
    }
}

const fn pos_to_index(block_pos: UVec3, dims: Dimensions) -> Option<BlockIndex> {
    if block_pos.x >= dims.x || block_pos.y >= dims.y || block_pos.z >= dims.z {
        None
    } else {
        Some((block_pos.y * (dims.x * dims.z) + block_pos.z * dims.x + block_pos.x) as BlockIndex)
    }
}

fn neighbor_of(mut block_pos: BlockPos, face: Face, dims: Dimensions) -> Option<BlockIndex> {
    match face {
        Face::Top => block_pos.y += 1,
        Face::Bottom => block_pos.y -= 1,
        Face::Right => block_pos.x += 1,
        Face::Left => block_pos.x -= 1,
        Face::Back => block_pos.z += 1,
        Face::Forward => block_pos.z -= 1,
    }
    pos_to_index(block_pos, dims)
}
