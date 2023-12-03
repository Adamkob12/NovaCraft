use crate::action::properties::DynamicProperty;
use crate::action::{BreakBlockGlobalEvent, PlaceBlockGlobalEvent};
use crate::blocks::{
    existence_conditions::*,
    meshreg::MeshRegistry,
    properties::{PassiveProperty, PhysicalProperty},
    BlockPropertyRegistry, WorldBlockUpdate,
};

use super::*;

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

pub fn handle_block_updates(
    mut world_block_update_events: EventReader<WorldBlockUpdate>,
    mut break_block_global_sender: EventWriter<BreakBlockGlobalEvent>,
    mut place_block_global_sender: EventWriter<PlaceBlockGlobalEvent>,
    mut commands: Commands,
    chunk_map: Res<ChunkMap>,
    passive_preg: Res<BlockPropertyRegistry<PassiveProperty>>,
    physical_preg: Res<BlockPropertyRegistry<PhysicalProperty>>,
    dyn_preg: Res<BlockPropertyRegistry<DynamicProperty>>,
    breg: Res<MeshRegistry>,
    grids: Query<(&Grid, &MainChild, &XSpriteChild), With<Chunk>>,
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
        let chunk_entity = chunk_map.pos_to_ent.get(&chunk_cords).unwrap();
        let (Grid(chunk_grid), MainChild(main_child), XSpriteChild(xsprite_child)) =
            grids.get(*chunk_entity).unwrap();
        let block = chunk_grid
            .read()
            .unwrap()
            .get_block(block_pos)
            .expect("block pos out of bounds");
        let (block_mesh, block_entity, block_mat) = match breg.get_mesh(&block) {
            VoxelMesh::NormalCube(mesh) => (mesh.clone(), main_child, &main_mat.0),
            VoxelMesh::XSprite(mesh) => (mesh.clone(), xsprite_child, &xsprite_mat.0),
            _ => continue,
        };
        let surrounding_blocks = chunk_grid
            .read()
            .unwrap()
            .get_neighbors_or(block_pos, Block::AIR)
            .map(|x| Some(x));

        let solver_data = ExistenceConditionSolverData { surrounding_blocks };
        for physical_property in physical_preg.get_properties(&block) {
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
                            block_mat.clone(),
                            global_pos,
                            physical_preg.get_density(&block),
                            block,
                        );
                    }
                }
            }
        }

        for dynamic_property in dyn_preg.get_properties(&block) {
            match dynamic_property {
                DynamicProperty::ExistenceCondition(cond) => {
                    break_block = !cond.solve(solver_data);
                }
                DynamicProperty::BlockTransformIf(cond, trans) => {
                    if cond.solve(solver_data) {
                        replace_with = Some(trans(block));
                    }
                }
            }
        }

        if break_block {
            break_block_global_sender
                .send(BreakBlockGlobalEvent::new(block_pos).with_chunk_entity(*block_entity))
        } else if let Some(alt) = replace_with {
            place_block_global_sender.send(PlaceBlockGlobalEvent {
                block: alt,
                chunk_cords,
                block_pos,
            })
        }
    }
}
