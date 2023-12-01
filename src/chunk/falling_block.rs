use bevy_xpbd_3d::prelude::ShapeHits;

use crate::action::{properties::FallingBlock, PlaceBlockGlobalEvent};

use super::*;

use crate::player::RigidLayer;
use crate::{action::VOXEL_DIMS, utils::to_global_pos};
use bevy_xpbd_3d::prelude::*;

pub fn follow_falling_block(
    mut commands: Commands,
    falling_blocks: Query<(Entity, &ShapeHits, &Block, &Transform, &FallingBlock)>,
    mut global_block_place_event_sender: EventWriter<PlaceBlockGlobalEvent>,
) {
    for (entity, hits, block, transform, FallingBlock { origin }) in falling_blocks.iter() {
        if !hits.is_empty() {
            let (chunk_pos, block_index, flag) =
                position_to_chunk_position(transform.translation + Vec3::Y * 0.1, CHUNK_DIMS);
            let block_index = one_d_cords(block_index, CHUNK_DIMS);
            if flag && block_index != *origin {
                info!(
                    "Falling block collision, at chunk: {:?} at position: {}, by block: {:?}",
                    chunk_pos, transform.translation, *block
                );
                global_block_place_event_sender.send(PlaceBlockGlobalEvent {
                    block: *block,
                    chunk_pos,
                    block_index,
                });
                commands.entity(entity).despawn();
            }
        }
    }
}

pub(super) fn spawn_falling_block(
    commands: &mut Commands,
    mesh_handle: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    index: usize,
    chunk_pos: ChunkCords,
    density: f32,
    block: Block,
) {
    info!("Spawned falling block: {:?}", block);
    let mut collider = Collider::cuboid(0.98, 0.98, 0.98);
    collider.set_scale(Vec3::ONE * 0.99, 10);
    let mut caster_shape = collider.clone();
    caster_shape.set_scale(Vec3::splat(0.99), 10);
    commands
        .spawn(PbrBundle {
            mesh: mesh_handle,
            material,
            transform: Transform::from_translation(to_global_pos(
                index,
                chunk_pos,
                VOXEL_DIMS.into(),
                CHUNK_DIMS,
            )),
            ..Default::default()
        })
        .insert(Friction::ZERO.with_combine_rule(CoefficientCombine::Min))
        .insert(Restitution::ZERO.with_combine_rule(CoefficientCombine::Min))
        .insert(GravityScale(2.4))
        .insert(CollisionLayers::new(
            [RigidLayer::FallingBlock],
            [RigidLayer::Ground],
        ))
        .insert(RigidBody::Dynamic)
        .insert(MassPropertiesBundle::new_computed(&collider, density))
        .insert(
            LockedAxes::ROTATION_LOCKED
                .lock_translation_x()
                .lock_translation_z(),
        )
        .insert(
            ShapeCaster::new(caster_shape, Vec3::ZERO, Quat::IDENTITY, Vec3::NEG_Y)
                .with_query_filter(
                    SpatialQueryFilter::new().with_masks([crate::player::RigidLayer::Ground]),
                )
                .with_max_time_of_impact(0.2),
        )
        .insert(collider)
        .insert(block)
        .insert(FallingBlock { origin: index });
}
