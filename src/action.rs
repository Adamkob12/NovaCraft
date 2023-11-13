#![allow(dead_code)]
pub use crate::blocks::*;
pub use crate::player::*;
pub use crate::prelude::*;

mod break_place_blocks;

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PrimeAction>().add_event::<SecondAction>();
        app.init_resource::<ActionKeyBinds>();
        app.add_systems(Update, broadcast_actions);
    }
}

#[derive(Resource)]
pub struct ActionKeyBinds {
    prime_action: MouseButton,
    second_action: MouseButton,
}

#[derive(Event)]
pub struct PrimeAction {
    // In millies from start
    time_stamp: u128,
}

#[derive(Event)]
pub struct SecondAction {
    // In millies from start
    time_stamp: u128,
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
    if buttons.just_pressed(prime_action_key) || buttons.just_released(prime_action_key) {
        prime_action.send(PrimeAction {
            time_stamp: time.elapsed().as_millis(),
        });
    }
    if buttons.just_pressed(second_action_key) || buttons.just_released(second_action_key) {
        second_action.send(SecondAction {
            time_stamp: time.elapsed().as_millis(),
        });
    }
}

impl Default for ActionKeyBinds {
    fn default() -> Self {
        ActionKeyBinds {
            prime_action: MouseButton::Left,
            second_action: MouseButton::Right,
        }
    }
}
