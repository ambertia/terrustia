use bevy::prelude::*;

use crate::player::Player;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_item_pickups)
            .add_event::<ItemPickedUp>();
    }
}

#[derive(Component, Default)]
/// Component to contain inventory information
// This only needs to hold an array of block id's for now because the only interactable blocks are
// the three types of foreground blocks, which are all stackable. This will change in the future
// and require more complex inventory management.
// Option should default to None which is perfect.
pub struct Inventory([Option<ItemStack>; 5]);

struct ItemStack {
    count: usize,
    item_id: usize,
}

#[derive(Event)]
pub struct ItemPickedUp(pub usize);

/// Process all pending ItemPickedUp events and modify the player's inventory accordingly
fn handle_item_pickups(
    mut events: EventReader<ItemPickedUp>,
    mut inventory: Single<&mut Inventory, With<Player>>,
) {
    for event in events.read() {
        let mut first_empty_slot: Option<usize> = None;
        // Iterate over all inventory slots
        for i in 0..(inventory.0.len()) {
            match &inventory.0[i] {
                // If the slot has a stack with matching item_id, put the item in this stack
                Some(s) if s.item_id == event.0 => {
                    inventory.0[i] = Some(ItemStack {
                        item_id: s.item_id,
                        count: s.count + 1,
                    });
                    return;
                }
                // Track the first empty inventory slot we find, if any
                None if first_empty_slot.is_none() => first_empty_slot = Some(i),
                _ => {}
            }
        }
        // If no such stack exists, put the item in the first empty slot
        if let Some(i) = first_empty_slot {
            inventory.0[i] = Some(ItemStack {
                item_id: event.0,
                count: 1,
            });
        }
    }
}
