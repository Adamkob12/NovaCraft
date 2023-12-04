// REFACTORED

//! Chunk meta-data
use super::*;

#[derive(Component)]
/// [`SubChunkMD`] means "Chunk Meta Data". It holds the metadata of a subchunk in an [`RwLock`].
pub struct SubChunkMD(pub RwLock<MetaData>);

/// Enum of all the possible subchunk metadatas
pub enum MetaData {
    CubeMD(MeshMD<Block>),
    XSpriteMD(XSpriteMetaData<Block>),
}

impl MetaData {
    /// Log the breaking of a block in the metadata.
    pub fn log_break(&mut self, block_pos: BlockPos, adj_blocks: [Option<Block>; 6]) {
        match self {
            Self::CubeMD(meshmd) => {
                meshmd.log(VoxelChange::Broken, block_pos, Block::STONE, adj_blocks)
            }
            Self::XSpriteMD(xspritemd) => {
                xspritemd
                    .log
                    .push((VoxelChange::Broken, Block::GREENERY, block_pos))
            }
        }
    }

    /// Log the placing of a block in a metadata.
    pub fn log_place(&mut self, block_pos: BlockPos, block: Block, adj_blocks: [Option<Block>; 6]) {
        match self {
            Self::CubeMD(meshmd) => meshmd.log(VoxelChange::Added, block_pos, block, adj_blocks),
            Self::XSpriteMD(xspritemd) => {
                xspritemd.log.push((VoxelChange::Added, block, block_pos))
            }
        }
    }

    /// Get the metadata of the cube subchunk.
    pub fn extract_meshmd(&self) -> Option<&MeshMD<Block>> {
        match self {
            Self::CubeMD(meshmd) => Some(meshmd),
            _ => None,
        }
    }

    /// Get the metadata of the cube subchunk as a mut ref.
    pub fn extract_meshmd_mut(&mut self) -> Option<&mut MeshMD<Block>> {
        match self {
            Self::CubeMD(meshmd) => Some(meshmd),
            _ => None,
        }
    }
}
