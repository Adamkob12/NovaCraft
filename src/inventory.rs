use crate::action::Block;
use crate::prelude::*;

pub const INVENTORY_SIZE: usize = 9;
pub const INVENTORY_SCALE: usize = 3;

pub struct InventoryPlugin;

#[derive(Resource)]
pub struct Inventory {
    pub(super) pack: [InventorySlot; INVENTORY_SIZE * INVENTORY_SCALE],
    pub(super) bar: [InventorySlot; INVENTORY_SIZE],
    pub(super) current: usize,
}

impl Inventory {
    pub(super) fn new() -> Inventory {
        Inventory::default()
    }

    pub(super) fn with_bar_slot(mut self, slot_index: usize, slot: InventorySlot) -> Inventory {
        self.bar[slot_index] = slot;
        self
    }

    pub(super) fn with_pack_slot(mut self, slot_index: usize, slot: InventorySlot) -> Inventory {
        self.pack[slot_index] = slot;
        self
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory {
            pack: [InventorySlot::Empty; INVENTORY_SIZE * INVENTORY_SCALE],
            bar: [InventorySlot::Empty; INVENTORY_SIZE],
            current: 0,
        }
    }
}

#[derive(Copy, Clone)]
pub(super) enum InventorySlot {
    Empty,
    Stack(Block, usize),
    Single(Block),
}

impl Default for InventorySlot {
    fn default() -> Self {
        Self::Empty
    }
}

impl InventorySlot {
    pub(super) fn get(&self) -> Option<Block> {
        match self {
            Self::Empty => None,
            Self::Single(item) | Self::Stack(item, _) => Some(*item),
        }
    }

    pub(super) fn take_amount(&mut self, amount_to_take: usize) -> Option<(Block, usize)> {
        match *self {
            Self::Empty => None,
            Self::Single(block) => {
                *self = Self::Empty;
                Some((block, 1))
            }
            Self::Stack(block, ref mut amount) => {
                if *amount > amount_to_take {
                    *amount -= amount_to_take;
                    return Some((block, amount_to_take));
                } else {
                    *self = Self::Empty;
                    return Some((block, amount_to_take));
                }
            }
        }
    }

    pub(super) fn take_single(&mut self) -> Option<Block> {
        self.take_amount(1).map(|(x, _)| x)
    }

    // ratio: 0.0 - 1.0
    #[allow(dead_code)]
    pub(super) fn take_ratio(&mut self, ratio: f32) -> Option<(Block, usize)> {
        match *self {
            Self::Empty => None,
            Self::Single(block) => {
                *self = Self::Empty;
                Some((block, 1))
            }
            Self::Stack(_, amount) => self.take_amount((amount as f32 * ratio) as usize),
        }
    }

    #[allow(dead_code)]
    pub(super) fn take_all(&mut self) -> Option<(Block, usize)> {
        self.take_ratio(1.0)
    }
}

impl Inventory {
    pub fn get_current(&self) -> Option<Block> {
        self.bar[self.current].get()
    }

    pub fn take_current_single(&mut self) -> Option<Block> {
        self.bar[self.current].take_single()
    }
}

#[derive(Resource)]
pub struct InventoryKeyBinds {
    pub open_inventory: KeyCode,
    pub next_item: KeyCode,
    pub prev_item: KeyCode,
    pub item1: KeyCode,
    pub item2: KeyCode,
    pub item3: KeyCode,
    pub item4: KeyCode,
    pub item5: KeyCode,
    pub item6: KeyCode,
    pub item7: KeyCode,
    pub item8: KeyCode,
    pub item9: KeyCode,
}

impl Default for InventoryKeyBinds {
    fn default() -> Self {
        InventoryKeyBinds {
            open_inventory: KeyCode::E,
            next_item: KeyCode::Q,
            prev_item: KeyCode::P,
            item1: KeyCode::Key2,
            item2: KeyCode::Key1,
            item3: KeyCode::R,
            item4: KeyCode::Z,
            item5: KeyCode::X,
            item6: KeyCode::V,
            item7: KeyCode::C,
            item8: KeyCode::G,
            item9: KeyCode::F,
        }
    }
}

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InventoryKeyBinds>();
        app.insert_resource(
            Inventory::new()
                .with_bar_slot(0, InventorySlot::Stack(Block::STONE, 20))
                .with_bar_slot(1, InventorySlot::Stack(Block::GRASS, 5))
                .with_bar_slot(2, InventorySlot::Stack(Block::GREENERY, 5))
                .with_pack_slot(0, InventorySlot::Single(Block::DIRT)),
        );
        app.add_systems(PreUpdate, inventory_input);
    }
}

fn inventory_input(
    mut inventory: ResMut<Inventory>,
    keybinds: Res<InventoryKeyBinds>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(keybinds.next_item) {
        inventory.current += 1;
        inventory.current %= INVENTORY_SIZE;
    }
    if keys.just_pressed(keybinds.prev_item) {
        inventory.current += INVENTORY_SIZE - 1;
        inventory.current %= INVENTORY_SIZE;
    }
    if keys.just_pressed(keybinds.item1) {
        inventory.current = 0 % INVENTORY_SIZE;
    }
    if keys.just_pressed(keybinds.item2) {
        inventory.current = 0 % INVENTORY_SIZE;
    }
    if keys.just_pressed(keybinds.item3) {
        inventory.current = 2 % INVENTORY_SIZE;
    }
    if keys.just_pressed(keybinds.item4) {
        inventory.current = 3 % INVENTORY_SIZE;
    }
    if keys.just_pressed(keybinds.item5) {
        inventory.current = 4 % INVENTORY_SIZE;
    }
    if keys.just_pressed(keybinds.item6) {
        inventory.current = 5 % INVENTORY_SIZE;
    }
    if keys.just_pressed(keybinds.item7) {
        inventory.current = 6 % INVENTORY_SIZE;
    }
    if keys.just_pressed(keybinds.item8) {
        inventory.current = 7 % INVENTORY_SIZE;
    }
    if keys.just_pressed(keybinds.item9) {
        inventory.current = 8 % INVENTORY_SIZE;
    }
}
