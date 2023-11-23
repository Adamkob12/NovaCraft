use crate::action::properties::FallingBlock;
use crate::action::BreakBlockGlobalEvent;
use crate::player::RigidLayer;
use crate::{
    action::{
        blockreg::BlockRegistry,
        properties::{BlockProperty, BlockPropertyRegistry},
        WorldBlockUpdate, VOXEL_DIMS,
    },
    utils::to_global_pos,
};
use bevy_xpbd_3d::prelude::*;

use super::*;

pub(super) fn handle_block_updates(
    mut world_block_update_events: EventReader<WorldBlockUpdate>,
    mut break_block_global_sender: EventWriter<BreakBlockGlobalEvent>,
    mut commands: Commands,
    chunk_map: Res<ChunkMap>,
    bpreg: Res<BlockPropertyRegistry>,
    breg: Res<BlockRegistry>,
    grids: Query<(&Grid, &MainChild, &XSpriteChild), With<Chunk>>,
    main_mat: Res<BlockMaterial>,
    xsprite_mat: Res<XSpriteMaterial>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for wbu in world_block_update_events.read() {
        let WorldBlockUpdate {
            chunk_pos,
            block_index,
            block_update: _,
        } = wbu;

        let chunk_entity = chunk_map.pos_to_ent.get(chunk_pos).unwrap();
        let (Grid(grid), MainChild(main_child), XSpriteChild(xsprite_child)) =
            grids.get(*chunk_entity).unwrap();
        let block = grid.read().unwrap()[*block_index];
        let (block_mesh, block_entity, block_mat) = match breg.get_mesh(&block) {
            VoxelMesh::NormalCube(mesh) => (mesh.clone(), main_child, &main_mat.0),
            VoxelMesh::CustomMesh(mesh) => (mesh.clone(), xsprite_child, &xsprite_mat.0),
            _ => continue,
        };
        let block_below = get_neighbor(*block_index, Bottom, CHUNK_DIMS)
            .map_or(Block::AIR, |i| grid.read().unwrap()[i]);
        for property in bpreg.iter_properties(&block) {
            match property {
                BlockProperty::AffectedByGravity => {
                    if bpreg.contains_property(&block_below, &BlockProperty::YieldToFallingBlock) {
                        break_block_global_sender.send(
                            BreakBlockGlobalEvent::new(*block_index)
                                .with_chunk_entity(*block_entity),
                        );
                        spawn_falling_block(
                            &mut commands,
                            meshes.add(block_mesh.clone()),
                            block_mat.clone(),
                            *block_index,
                            *chunk_pos,
                            bpreg.get_density(&block),
                            block,
                        );
                    }
                }
                BlockProperty::MustBeOnTopOf(block) => {
                    if block_below != *block {
                        break_block_global_sender.send(
                            BreakBlockGlobalEvent::new(*block_index)
                                .with_chunk_entity(*block_entity),
                        )
                    }
                }
                _ => {}
            }
        }
    }
}

fn spawn_falling_block(
    commands: &mut Commands,
    mesh_handle: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    index: usize,
    chunk_pos: ChunkCords,
    density: f32,
    block: Block,
) {
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
        .insert(FallingBlock);
}
