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

pub fn adj_blocks(index: usize, chunk_pos: [i32; 2], dims: Dimensions) -> Vec<(usize, [i32; 2])> {
    let mut to_return = vec![];
    for i in 0..6 {
        let face = Face::from(i);
        if let Some(neighbor) = get_neighbor(index, face, dims) {
            to_return.push((neighbor, chunk_pos));
        } else if let Some(neighbor) = get_neigbhor_across_chunk_safe(dims, index, face) {
            let chunk = to_cords(Some(face.into()));
            to_return.push((neighbor, [chunk_pos[0] + chunk[0], chunk_pos[1] + chunk[1]]));
        }
    }
    to_return
}

pub fn three_d_cords_vec3(index: usize, dims: Dimensions) -> Vec3 {
    let tmp = three_d_cords_arr(index, dims);
    Vec3::new(tmp[0] as f32, tmp[1] as f32, tmp[2] as f32)
}

pub fn to_global_pos(index: usize, cords: [i32; 2], voxel_dims: Vec3, dims: Dimensions) -> Vec3 {
    let Vec3 { x, y, z } = three_d_cords_vec3(index, dims);
    let [u, v] = [
        cords[0] as f32 * dims.0 as f32,
        cords[1] as f32 * dims.2 as f32,
    ];
    Vec3::new(x + u, y, z + v) * voxel_dims /*  + voxel_dims / 2.0 */
}
