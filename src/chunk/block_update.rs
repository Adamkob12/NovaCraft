use crate::player::RigidLayer;
use crate::{
    action::{
        blockreg::BlockRegistry,
        properties::{BlockProperty, BlockPropertyRegistry},
        BlockBreakEvent, WorldBlockUpdate, VOXEL_DIMS,
    },
    utils::to_global_pos,
};
use bevy_xpbd_3d::prelude::*;

use super::*;

pub(super) fn handle_block_updates(
    mut world_block_update_events: EventReader<WorldBlockUpdate>,
    mut break_block_event_sender: EventWriter<BlockBreakEvent>,
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
                        break_block_event_sender.send(BlockBreakEvent(*block_entity, *block_index));
                        spawn_falling_block(
                            &mut commands,
                            meshes.add(block_mesh.clone()),
                            &block_mesh,
                            block_mat.clone(),
                            *block_index,
                            *chunk_pos,
                            bpreg.get_density(&block),
                        );
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
    mesh: &Mesh,
    material: Handle<StandardMaterial>,
    index: usize,
    chunk_pos: ChunkCords,
    density: f32,
) {
    let collider = Collider::trimesh_from_mesh(mesh).unwrap();
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
        .insert(collider);
}
