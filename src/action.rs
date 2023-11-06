pub use crate::blocks::*;
use crate::chunk::ChunkCords;
pub use crate::player::*;
pub use crate::prelude::*;

mod break_place_blocks;

pub struct ActionPlugin;

// impl Plugin for ActionPlugin {
//     fn build(&self, app: &mut App) {}
// }
//
#[derive(Event)]
pub struct BlockBreakEvent(Vec<(Block, usize, ChunkCords)>);

#[derive(Event)]
pub struct BlockPlaceEvent(Vec<(Block, usize, ChunkCords, Face)>);

// fn handle_actions(
//     mut block_break_writer: EventWriter<BlockBreakEvent>,
//     mut block_place_writer: EventWriter<BlockPlaceEvent>,
//
//     buttons: Res<Input<MouseButton>>,
// ) {
// }
