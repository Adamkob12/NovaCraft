use crate::chunk::ChunkCords;
use crate::prelude::*;

pub fn get_neighboring_chunk_cords(cords: ChunkCords, dir: Direction) -> ChunkCords {
    match dir {
        East => [cords[0] + 1, cords[1]],
        West => [cords[0] - 1, cords[1]],
        North => [cords[0], cords[1] + 1],
        South => [cords[0], cords[1] - 1],
        NoEast => [cords[0] + 1, cords[1] + 1],
        NoWest => [cords[0] - 1, cords[1] + 1],
        SoEast => [cords[0] + 1, cords[1] - 1],
        SoWest => [cords[0] - 1, cords[1] - 1],
    }
}

// get distance between chunks
pub fn chunk_distance(cords1: ChunkCords, cords2: ChunkCords) -> i32 {
    (cords1[0] - cords2[0])
        .abs()
        .max((cords1[1] - cords2[1]).abs())
}
