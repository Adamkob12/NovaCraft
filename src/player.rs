pub mod controller;
pub mod movement;

use crate::chunk::CurrentChunk;
use crate::chunk::HEIGHT;
use crate::chunk::RENDER_DISTANCE;
use crate::chunk::WIDTH;
use crate::mesh_utils::Chunk;
use crate::mesh_utils::ComputeChunk;
use crate::prelude::*;
use bevy::ecs::event::ManualEventReader;
use bevy::input::mouse::MouseMotion;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy::{
    core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
    pbr::{
        ScreenSpaceAmbientOcclusionBundle, ScreenSpaceAmbientOcclusionQualityLevel,
        ScreenSpaceAmbientOcclusionSettings,
    },
};
use bevy_xpbd_3d::prelude::Collider;
use bevy_xpbd_3d::prelude::*;
pub use controller::*;
use movement::*;

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
            sensitivity: 0.000037,
            speed: 30.,
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

/// Spawns the `Camera3dBundle` to be controlled
pub(super) fn setup_player(mut commands: Commands) {
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(
                    // -RENDER_DISTANCE as f32 * WIDTH as f32,
                    0.0,
                    HEIGHT as f32 * 2.0, // -RENDER_DISTANCE as f32 * LENGTH as f32,
                    0.0,
                )
                .looking_to(Vec3::new(5.0, -1.0, 5.0), Vec3::Y),
                ..Default::default()
            },
            PlayerCamera,
        ))
        .insert(FogSettings {
            color: Color::rgb(0.85, 0.95, 1.0),
            falloff: FogFalloff::Linear {
                start: ((RENDER_DISTANCE - 2) * WIDTH as i32) as f32,
                end: ((RENDER_DISTANCE - 1) * WIDTH as i32) as f32,
            },
            ..Default::default()
        })
        .insert(CharacterControllerBundle::new(Collider::cuboid(
            0.8, 3.4, 0.8,
        )))
        .insert(Friction::ZERO.with_combine_rule(CoefficientCombine::Min))
        .insert(Restitution::ZERO.with_combine_rule(CoefficientCombine::Min))
        .insert(GravityScale(2.0))
        .insert(TemporalAntiAliasBundle::default())
        .insert(ScreenSpaceAmbientOcclusionBundle {
            settings: ScreenSpaceAmbientOcclusionSettings {
                quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
            },
            ..Default::default()
        });
}

/// Used in queries when you want flycams and not other cameras
/// A marker component used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct PlayerCamera;

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
        app.add_plugins((TemporalAntiAliasPlugin,))
            .init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .init_resource::<KeyBindings>()
            .insert_resource(CurrentChunk([0, 0]))
            .add_systems(
                Update,
                setup_player.run_if(
                    not(any_with_component::<ComputeChunk>())
                        .and_then(not(any_with_component::<PlayerCamera>()))
                        .and_then(any_with_component::<Chunk>()),
                ),
            )
            .add_systems(Startup, initial_grab_cursor)
            .add_systems(
                Update,
                (
                    // update_current_chunk,
                    player_look,
                    cursor_grab,
                ), /*   .run_if(in_state(InitialChunkLoadState::Complete)), */
            );
    }
}
//
// fn update_current_chunk(
//     mut current_chunk: ResMut<CurrentChunk>,
//     player: Query<&Transform, With<PlayerCamera>>,
// ) {
//     if let Ok(t) = player.get_single() {
//         let tmp = position_to_chunk(t.translation, CHUNK_DIMS);
//         if tmp != current_chunk.0 {
//             current_chunk.0 = tmp;
//             dbg!(tmp);
//         }
//     }
// }
