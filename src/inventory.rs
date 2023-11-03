use crate::blocks::BlockId;
use crate::prelude::*;

pub const INVENTORY_SIZE: usize = 10;
pub const INVENTORY_SCALE: usize = 3;

#[derive(Resource)]
pub struct Inventory {
    pub pack: [BlockId; INVENTORY_SIZE * INVENTORY_SCALE],
    pub bar: [BlockId; INVENTORY_SIZE],
}
