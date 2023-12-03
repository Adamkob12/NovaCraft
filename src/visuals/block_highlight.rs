use super::*;
use crate::chunk::*;
use crate::prelude::*;
use std::f32::consts::PI;

const HIGHLIGHT_SCALE: f32 = 1.005;

pub(super) fn highlight_target_block(target_block: Res<TargetBlock>, mut gizmos: Gizmos) {
    if target_block.ignore_flag == true {
        return;
    }
    // convert to Vec3 and offset
    let tmp = target_block.block_pos.as_vec3();
    // offset the chunk dims
    let block_pos = tmp
        + Vec3::new(
            target_block.chunk_cords[0] as f32 * WIDTH as f32,
            0.0,
            target_block.chunk_cords[1] as f32 * LENGTH as f32,
        );

    let offset = target_block.ray_direction.normalize_or_zero() * (-HIGHLIGHT_SCALE / 100.0);
    gizmos.cuboid(
        Transform::from_translation(block_pos + offset)
            .with_scale(Vec3::from(crate::blocks::VOXEL_DIMS) * HIGHLIGHT_SCALE),
        Color::BLACK,
    );
}

pub(super) fn config_gizmos(mut gizmos_config: ResMut<GizmoConfig>) {
    gizmos_config.line_width = PI / 1.35;
}
