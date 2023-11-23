use crate::action::BreakBlockGlobalEvent;
use crate::blocks::{
    blockreg::BlockRegistry,
    existence_conditions::*,
    properties::{BlockProperty, BlockPropertyRegistry, PassiveProperty, PhysicalProperty},
    WorldBlockUpdate,
};

use super::*;

pub(super) fn handle_block_updates(
    mut world_block_update_events: EventReader<WorldBlockUpdate>,
    mut break_block_global_sender: EventWriter<BreakBlockGlobalEvent>,
    mut commands: Commands,
    chunk_map: Res<ChunkMap>,
    exconds: Res<ExistenceConditions>,
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
                BlockProperty::Physical(PhysicalProperty::AffectedByGravity) => {
                    if bpreg.contains_property(
                        &block_below,
                        &BlockProperty::Passive(PassiveProperty::YieldToFallingBlock),
                    ) {
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
                // BlockProperty::ConditionalExistence(ConditionalExistence::BlockUnderMust(cond)) => {
                //     if !cond(&block_below) {
                //         break_block_global_sender.send(
                //             BreakBlockGlobalEvent::new(*block_index)
                //                 .with_chunk_entity(*block_entity),
                //         )
                //     }
                // }
                _ => {}
            }
        }
        let mut break_block = false;
        match exconds.get_condition(&block) {
            // The block always exists
            ConditionalExistence::Always => {}
            // If the block can never exist, we break it.
            ConditionalExistence::Never => break_block = true,
            ConditionalExistence::BlockUnderMust(cond) => {
                if !cond(block_below) {
                    break_block = true
                }
            }
            // TODO: support chained conditions (ie: ConditionalExistence::AND / OR)
            _ => {}
        }
        if break_block {
            break_block_global_sender
                .send(BreakBlockGlobalEvent::new(*block_index).with_chunk_entity(*block_entity))
        }
    }
}
