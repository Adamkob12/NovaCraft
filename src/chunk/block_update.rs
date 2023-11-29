use crate::action::properties::DynamicProperty;
use crate::action::{BreakBlockGlobalEvent, PlaceBlockGlobalEvent};
use crate::blocks::{
    blockreg::BlockRegistry,
    existence_conditions::*,
    properties::{PassiveProperty, PhysicalProperty},
    BlockPropertyRegistry, WorldBlockUpdate,
};

use super::*;

#[derive(Resource, Default, PartialEq)]
pub struct ChunkUpdateLock(bool);

#[rustfmt::skip]
impl ChunkUpdateLock {
    pub fn lock(&mut self) { self.0 = false; }
    pub fn unlock(&mut self) { self.0 = true; }
    pub fn _locked() -> Self { Self(false) }
    pub fn unlocked() -> Self { Self(true) }
    pub fn is_locked(&self) -> bool { !self.0 }
    pub fn is_unlocked(&self) -> bool { self.0 }
}

pub fn handle_block_updates(
    mut world_block_update_events: EventReader<WorldBlockUpdate>,
    mut break_block_global_sender: EventWriter<BreakBlockGlobalEvent>,
    mut place_block_global_sender: EventWriter<PlaceBlockGlobalEvent>,
    mut commands: Commands,
    chunk_map: Res<ChunkMap>,
    passive_preg: Res<BlockPropertyRegistry<PassiveProperty>>,
    physical_preg: Res<BlockPropertyRegistry<PhysicalProperty>>,
    dyn_preg: Res<BlockPropertyRegistry<DynamicProperty>>,
    breg: Res<BlockRegistry>,
    grids: Query<(&Grid, &MainChild, &XSpriteChild), With<Chunk>>,
    main_mat: Res<BlockMaterial>,
    xsprite_mat: Res<XSpriteMaterial>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut update_lock: ResMut<ChunkUpdateLock>,
) {
    if world_block_update_events.is_empty() {
        update_lock.unlock();
    } else if update_lock.is_unlocked() {
        update_lock.lock();
    }

    for wbu in world_block_update_events.read() {
        let mut break_block = false;
        let mut replace_with = None;
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
        let block_above = get_neighbor(*block_index, Top, CHUNK_DIMS)
            .map_or(Block::AIR, |i| grid.read().unwrap()[i]);
        for physical_property in physical_preg.get_properties(&block) {
            match physical_property {
                PhysicalProperty::AffectedByGravity => {
                    if passive_preg
                        .contains_property(&block_below, &PassiveProperty::YieldToFallingBlock)
                    {
                        break_block = true;
                        spawn_falling_block(
                            &mut commands,
                            meshes.add(block_mesh.clone()),
                            block_mat.clone(),
                            *block_index,
                            *chunk_pos,
                            physical_preg.get_density(&block),
                            block,
                        );
                    }
                }
            }
        }

        for dynamic_property in dyn_preg.get_properties(&block) {
            match dynamic_property {
                DynamicProperty::BlockAbove(trans) => {
                    let tmp = trans(block_above);
                    if tmp != block {
                        replace_with = Some(tmp);
                    }
                }
                DynamicProperty::ExistenceCondition(cond) => {
                    match cond {
                        // The block always exists
                        ExistenceCondition::Always => {}
                        // If the block can never exist, we break it.
                        ExistenceCondition::Never => break_block = true,
                        ExistenceCondition::BlockUnderMust(cond) => {
                            if !cond(block_below) {
                                break_block = true
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if break_block {
            break_block_global_sender
                .send(BreakBlockGlobalEvent::new(*block_index).with_chunk_entity(*block_entity))
        } else if let Some(alt) = replace_with {
            place_block_global_sender.send(PlaceBlockGlobalEvent {
                block: alt,
                chunk_pos: *chunk_pos,
                block_index: *block_index,
            })
        }
    }
}
