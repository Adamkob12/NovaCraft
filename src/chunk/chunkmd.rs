// Chunk meta-data
use super::*;

#[derive(Component)]
// CMMD = Chunk Mesh Meta Data
pub struct CMMD(pub RwLock<ChunkMD>);

pub enum ChunkMD {
    CubeMD(MeshMD<Block>),
    XSpriteMD(XSpriteMetaData<Block>),
}

impl ChunkMD {
    pub fn log_break(&mut self, index: usize, adj_blocks: [Option<Block>; 6]) {
        match self {
            Self::CubeMD(meshmd) => {
                meshmd.log(VoxelChange::Broken, index, Block::STONE, adj_blocks)
            }
            Self::XSpriteMD(xspritemd) => {
                xspritemd
                    .log
                    .push((VoxelChange::Broken, Block::GREENERY, index))
            }
        }
    }

    pub fn log_place(&mut self, index: usize, block: Block, adj_blocks: [Option<Block>; 6]) {
        match self {
            Self::CubeMD(meshmd) => meshmd.log(VoxelChange::Added, index, block, adj_blocks),
            Self::XSpriteMD(xspritemd) => xspritemd.log.push((VoxelChange::Added, block, index)),
        }
    }

    pub fn extract_meshmd(&self) -> Option<&MeshMD<Block>> {
        match self {
            Self::CubeMD(meshmd) => Some(meshmd),
            _ => None,
        }
    }

    pub fn extract_meshmd_mut(&mut self) -> Option<&mut MeshMD<Block>> {
        match self {
            Self::CubeMD(meshmd) => Some(meshmd),
            _ => None,
        }
    }
}
