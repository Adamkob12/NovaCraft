pub use crate::blocks::*;
use crate::chunk::{
    block_update::handle_block_updates, follow_falling_block, ChunkCords, CHUNK_DIMS,
};
use crate::inventory::Inventory;
pub use crate::player::*;
pub use crate::prelude::*;
use crate::utils::adj_blocks;

mod action_utils;
mod break_blocks;
mod place_blocks;

use action_utils::*;
pub use break_blocks::*;
pub use place_blocks::*;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PrimeAction>()
            .add_event::<SecondAction>()
            .add_event::<BlockPlaceEvent>()
            .add_event::<PlaceBlockGlobalEvent>()
            .add_event::<BreakBlockGlobalEvent>()
            .init_resource::<ActionKeyBinds>()
            .add_systems(
                PreUpdate,
                (
                    broadcast_actions,
                    sort_actions,
                    follow_falling_block,
                    (handle_place_block_event, global_block_breaker),
                    global_block_placer,
                    handle_block_updates,
                    apply_deferred,
                )
                    .run_if(any_with_component::<PlayerCamera>())
                    .chain(),
            );
    }
}

#[derive(Event)]
pub enum ActionType {
    Start,
    Stop,
}

#[derive(Resource)]
pub struct ActionKeyBinds {
    prime_action: MouseButton,
    second_action: MouseButton,
}

#[derive(Event)]
pub struct PrimeAction {
    // In millies from start
    #[allow(dead_code)]
    time_stamp: u128,
    action_type: ActionType,
}

#[derive(Event)]
pub struct SecondAction {
    // In millies from start
    #[allow(dead_code)]
    time_stamp: u128,
    action_type: ActionType,
}

fn broadcast_actions(
    mut prime_action: EventWriter<PrimeAction>,
    mut second_action: EventWriter<SecondAction>,
    time: Res<Time<Virtual>>,
    buttons: Res<Input<MouseButton>>,
    action_binds: Res<ActionKeyBinds>,
) {
    let prime_action_key = action_binds.prime_action;
    let second_action_key = action_binds.second_action;
    if buttons.just_pressed(prime_action_key) {
        prime_action.send(PrimeAction {
            time_stamp: time.elapsed().as_millis(),
            action_type: ActionType::Start,
        });
    }
    if buttons.just_released(prime_action_key) {
        prime_action.send(PrimeAction {
            time_stamp: time.elapsed().as_millis(),
            action_type: ActionType::Stop,
        });
    }
    if buttons.just_pressed(second_action_key) {
        second_action.send(SecondAction {
            time_stamp: time.elapsed().as_millis(),
            action_type: ActionType::Start,
        });
    }
    if buttons.just_released(second_action_key) {
        second_action.send(SecondAction {
            time_stamp: time.elapsed().as_millis(),
            action_type: ActionType::Stop,
        });
    }
}

fn sort_actions(
    target_block: Res<TargetBlock>,
    mut prime_action_reader: EventReader<PrimeAction>,
    mut second_action_reader: EventReader<SecondAction>,
    mut break_block_global_sender: EventWriter<BreakBlockGlobalEvent>,
    mut place_block_writer: EventWriter<BlockPlaceEvent>,
    inv: Res<Inventory>,
) {
    for prime_action in prime_action_reader.read() {
        if matches!(prime_action.action_type, ActionType::Start)
            && target_block.ignore_flag == false
        {
            break_block_global_sender.send(BreakBlockGlobalEvent {
                chunk_entity: Some(target_block.target_entity),
                chunk_pos: None,
                block_index: target_block.block_index,
            });
        }
    }
    for second_action in second_action_reader.read() {
        if matches!(second_action.action_type, ActionType::Start)
            && target_block.ignore_flag == false
        {
            if target_block.face_hit.is_none() {
                continue;
            }
            if let Some(block) = inv.get_current() {
                place_block_writer.send(BlockPlaceEvent(
                    target_block.target_entity,
                    target_block.block_index,
                    target_block.face_hit.unwrap(),
                    block,
                ));
            }
        }
    }
}

pub fn send_world_updates_surrounding_blocks(
    block_index: usize,
    chunk_pos: ChunkCords,
    world_block_update_sender: &mut EventWriter<WorldBlockUpdate>,
    block_update: BlockUpdate,
) {
    for (adj_block_index, adj_block_chunk) in adj_blocks(block_index, chunk_pos, CHUNK_DIMS) {
        world_block_update_sender.send(WorldBlockUpdate {
            block_index: adj_block_index,
            chunk_pos: adj_block_chunk,
            block_update: None,
        });
    }
    world_block_update_sender.send(WorldBlockUpdate {
        block_index,
        chunk_pos,
        block_update: Some(block_update),
    });
}

impl Default for ActionKeyBinds {
    fn default() -> Self {
        ActionKeyBinds {
            prime_action: MouseButton::Left,
            second_action: MouseButton::Right,
        }
    }
}
