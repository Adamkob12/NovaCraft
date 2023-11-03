use crate::chunk::ChunkCords;
use crate::prelude::*;

pub fn get_neighboring_chunk_cords(cords: ChunkCords, face: Face) -> ChunkCords {
    match face {
        Right => [cords[0] + 1, cords[1]],
        Left => [cords[0] - 1, cords[1]],
        Back => [cords[0], cords[1] + 1],
        Forward => [cords[0], cords[1] - 1],
        _ => panic!("Stacking chunks vertically not supported."),
    }
}

// get distance between chunks
pub fn chunk_distance(cords1: ChunkCords, cords2: ChunkCords) -> i32 {
    (cords1[0] - cords2[0])
        .abs()
        .max((cords1[1] - cords2[1]).abs())
}
