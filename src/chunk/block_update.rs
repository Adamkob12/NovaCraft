// REFACTORED

use crate::action::properties::DynamicProperty;
use crate::action::{BreakBlockGlobalEvent, PlaceBlockGlobalEvent};
use crate::blocks::{
    existence_conditions::*,
    meshreg::MeshRegistry,
    properties::{PassiveProperty, PhysicalProperty},
    BlockPropertyRegistry, WorldBlockUpdate,
};

use super::*;

/// If this resource is [`locked (self.0 == false)`](LockChunkUpdate::0) the chunks' won't update.
#[derive(Resource, Default, PartialEq)]
pub struct LockChunkUpdate(bool);

#[rustfmt::skip]
impl LockChunkUpdate {
    pub fn lock(&mut self) { self.0 = false; }
    pub fn unlock(&mut self) { self.0 = true; }
    pub fn locked() -> Self { Self(false) }
    pub fn unlocked() -> Self { Self(true) }
    pub fn is_locked(&self) -> bool { !self.0 }
    pub fn is_unlocked(&self) -> bool { self.0 }
}

/// This system runs once every frame. It processes all of the pending [`WorldBlockUpdate`] and
/// handles them. For example:
/// [`Block::DIRT`] has been broken --> [`WorldBlockUpdate`] broadcast in that position, and the
/// directly adjecant positions (block above, block below, etc.) --> [`WorldBlockUpdate`] has been
/// recieved in the same position where a [`Block::SAND`] is --> Iterate over the block's
/// properties --> catch that he has [`PhysicalProperty::AffectedByGravity`] --> handle
/// accordingly (spawn sand block that is affected by gravity etc.)
pub fn handle_block_updates(
    mut world_block_update_events: EventReader<WorldBlockUpdate>,
    mut break_block_global_sender: EventWriter<BreakBlockGlobalEvent>,
    mut place_block_global_sender: EventWriter<PlaceBlockGlobalEvent>,
    mut commands: Commands,
    chunk_map: Res<ChunkMap>,
    passive_preg: Res<BlockPropertyRegistry<PassiveProperty>>,
    physical_preg: Res<BlockPropertyRegistry<PhysicalProperty>>,
    dyn_preg: Res<BlockPropertyRegistry<DynamicProperty>>,
    mreg: Res<MeshRegistry>,
    grids: Query<(&Grid, &CubeChild, &XSpriteChild), With<ParentChunk>>,
    main_mat: Res<BlockMaterial>,
    xsprite_mat: Res<XSpriteMaterial>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for wbu in world_block_update_events.read() {
        let mut break_block = false;
        let mut replace_with = None;
        let WorldBlockUpdate {
            chunk_cords,
            block_pos,
            block_update: _,
        } = *wbu;

        let global_pos = BlockGlobalPos::new(block_pos, chunk_cords);
        let update_chunk_entity = chunk_map.pos_to_ent.get(&chunk_cords).unwrap();
        let (Grid(chunk_grid), CubeChild(cube_child), XSpriteChild(xsprite_child)) =
            grids.get(*update_chunk_entity).unwrap();
        let block_to_update = chunk_grid
            .read()
            .unwrap()
            .get_block(block_pos)
            .expect("block pos out of bounds");
        let (block_mesh, subchunk_entity, block_material) = match mreg.get_mesh(&block_to_update) {
            VoxelMesh::NormalCube(mesh) => (mesh.clone(), cube_child, &main_mat.0),
            VoxelMesh::XSprite(mesh) => (mesh.clone(), xsprite_child, &xsprite_mat.0),
            _ => continue,
        };
        let surrounding_blocks = chunk_grid
            .read()
            .unwrap()
            .get_neighbors_or(block_pos, Block::AIR)
            .map(|x| Some(x));

        // define the data the solver will need
        let solver_data = ExistenceConditionSolverData { surrounding_blocks };
        // handle physical properties
        for physical_property in physical_preg.get_properties(&block_to_update) {
            match physical_property {
                PhysicalProperty::AffectedByGravity => {
                    if passive_preg.contains_property(
                        &surrounding_blocks[Face::Bottom as usize].unwrap(),
                        &PassiveProperty::YieldToFallingBlock,
                    ) {
                        break_block = true;
                        spawn_falling_block(
                            &mut commands,
                            meshes.add(block_mesh.clone()),
                            block_material.clone(),
                            global_pos,
                            BlockPropertyRegistry::<PhysicalProperty>::get_density(
                                &block_to_update,
                            ),
                            block_to_update,
                        );
                    }
                }
            }
        }
        // handle dynamic properties
        for dynamic_property in dyn_preg.get_properties(&block_to_update) {
            match dynamic_property {
                DynamicProperty::ExistenceCondition(cond) => {
                    // if the solver (ExistenceCondition::solve()) returns false, that means that
                    // the block "cant exist" in the current position anymore.
                    break_block = !cond.solve(solver_data);
                }
                DynamicProperty::BlockTransformIf(cond, trans) => {
                    // if the condition evaluates to true, apply the transformation.
                    if cond.solve(solver_data) {
                        replace_with = Some(trans(block_to_update));
                    }
                }
            }
        }

        // handle the cases where the block needed to be broken / transformed.
        if break_block {
            break_block_global_sender.send(BreakBlockGlobalEvent::from_entity_and_pos(
                block_pos,
                *subchunk_entity,
            ))
        } else if let Some(alt) = replace_with {
            place_block_global_sender.send(PlaceBlockGlobalEvent {
                block: alt,
                chunk_cords,
                block_pos,
            })
        }
    }
}
