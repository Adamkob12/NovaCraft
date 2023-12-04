// REFACTORED

use super::*;
use crate::chunk::*;
use crate::prelude::*;
use std::f32::consts::PI;

const HIGHLIGHT_SCALE: f32 = 1.005;
const GIZMOS_LINE_WIDTH: f32 = PI / 1.25;

pub(super) fn highlight_target_block(target_block: Res<TargetBlock>, mut gizmos: Gizmos) {
    // The ignore flag might be on for several reasons, if its on, we ignore.
    if target_block.ignore_flag == true {
        return;
    }
    // convert to Vec3
    let tmp = target_block.block_pos.as_vec3();
    let block_pos = tmp
        + Vec3::new(
            target_block.chunk_cords[0] as f32 * WIDTH as f32,
            0.0,
            target_block.chunk_cords[1] as f32 * LENGTH as f32,
        );

    // The offset is meant to move the the gizmos cuboid slightly towards the player, because there may be
    // cases where the ground is covering a small part of the cuboid. This way its bold and clear.
    let offset = target_block.ray_direction.normalize_or_zero() * (-HIGHLIGHT_SCALE / 100.0);
    gizmos.cuboid(
        Transform::from_translation(block_pos + offset)
            .with_scale(Vec3::from(crate::blocks::VOXEL_DIMS) * HIGHLIGHT_SCALE),
        Color::BLACK,
    );
}

/// configure the gizmos
pub(super) fn config_gizmos(mut gizmos_config: ResMut<GizmoConfig>) {
    gizmos_config.line_width = GIZMOS_LINE_WIDTH;
}
