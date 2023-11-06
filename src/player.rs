pub mod cage;
pub mod movement;

use crate::blocks::Block;
use crate::chunk::ChunkCloseToPlayer;
use crate::chunk::CurrentChunk;
use crate::chunk::CHUNK_DIMS;
use crate::prelude::*;
use bevy::ecs::event::{Events, ManualEventReader};
use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    pbr::{
        ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionQualityLevel,
        ScreenSpaceAmbientOcclusionSettings,
    },
    prelude::*,
    render::camera::TemporalJitter,
};
use cage::*;
use movement::*;
// ALWAYS ODD!

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Resource, Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

/// Mouse sensitivity and movement speed
#[derive(Resource)]
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 40.,
        }
    }
}

/// Key configuration
#[derive(Resource)]
pub struct KeyBindings {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub move_ascend: KeyCode,
    pub move_descend: KeyCode,
    pub toggle_grab_cursor: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::W,
            move_backward: KeyCode::S,
            move_left: KeyCode::A,
            move_right: KeyCode::D,
            move_ascend: KeyCode::Space,
            move_descend: KeyCode::ShiftLeft,
            toggle_grab_cursor: KeyCode::Escape,
        }
    }
}

/// Used in queries when you want flycams and not other cameras
/// A marker component used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct FlyCam;

// Keeps track of the blocks surrounding the player for physics
/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

fn cursor_grab(
    keys: Res<Input<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        if keys.just_pressed(key_bindings.toggle_grab_cursor) {
            toggle_grab_cursor(&mut window);
        }
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        assert!(CAGE_SIZE % 2 == 1, "Cage size should always be odd!");
        app.add_plugins((TemporalAntiAliasPlugin,))
            .init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .init_resource::<KeyBindings>()
            .insert_resource(CurrentChunk([0, 0]))
            .add_systems(Startup, setup_player)
            .add_systems(Startup, initial_grab_cursor)
            .add_systems(
                Update,
                (player_move, player_look, cursor_grab, update_cage)
                  /*   .run_if(in_state(InitialChunkLoadState::Complete)), */
            );
    }
}
