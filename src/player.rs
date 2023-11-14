pub mod controller;
pub mod movement;

use crate::chunk::HEIGHT;
use crate::chunk::RENDER_DISTANCE;
use crate::chunk::WIDTH;
use crate::chunk::{CurrentChunk, CHUNK_DIMS};
use crate::mesh_utils::Chunk;
use crate::mesh_utils::ChunkCords;
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

pub const MAX_INTERACTION_DISTANCE: f32 = 7.0;
pub const SMALL_TRAVERSE: f32 = 0.005;

#[derive(Component)]
pub struct PlayerCamera;

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Resource, Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

#[derive(Component)]
pub struct PhysicalPlayer;

#[derive(Resource)]
pub struct TargetBlock {
    pub target_entity: Entity,
    pub chunk_cords: ChunkCords,
    pub block_index: usize,
    pub face_hit: Option<Face>,
    pub ray_direction: Vec3,
}

/// Mouse sensitivity and movement speed
#[derive(Resource)]
pub struct MovementSettings {
    pub sensitivity: f32,
}

#[derive(PhysicsLayer)]
pub enum RigidLayer {
    Player,
    Ground,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.000022,
        }
    }
}

/// Spawns the `Camera3dBundle` to be controlled
pub(super) fn setup_player(mut commands: Commands) {
    let player_entity = commands
        .spawn(PhysicalPlayer)
        .insert(SpatialBundle {
            transform: Transform::from_xyz(0.0, HEIGHT as f32 + 5.0, 0.0),
            ..Default::default()
        })
        .insert(CharacterControllerBundle::new(Collider::capsule(1.0, 0.4)))
        .insert(Friction::new(0.2).with_combine_rule(CoefficientCombine::Min))
        .insert(Restitution::ZERO.with_combine_rule(CoefficientCombine::Min))
        .insert(GravityScale(2.5))
        .insert(CollisionLayers::new(
            [RigidLayer::Player],
            [RigidLayer::Ground],
        ))
        .id();
    let camera_entity = commands
        .spawn(Camera3dBundle::default())
        .insert(PlayerCamera)
        .insert(TemporalAntiAliasBundle::default())
        .insert(ScreenSpaceAmbientOcclusionBundle {
            settings: ScreenSpaceAmbientOcclusionSettings {
                quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
            },
            ..Default::default()
        })
        .insert(FogSettings {
            color: Color::rgb(0.85, 0.95, 1.0),
            falloff: FogFalloff::Linear {
                start: ((RENDER_DISTANCE - 2) * WIDTH as i32) as f32,
                end: ((RENDER_DISTANCE - 1) * WIDTH as i32) as f32,
            },
            ..Default::default()
        })
        .id();
    commands
        .entity(player_entity)
        .push_children(&[camera_entity]);
}

fn update_target_block(
    mut target_block: ResMut<TargetBlock>,
    camera_rotation_transform: Query<&Transform, With<PlayerCamera>>,
    camera_position_transform: Query<&Transform, With<PhysicalPlayer>>,
    spatial_query: SpatialQuery,
) {
    // let tran = camera_query.get_single().unwrap();
    if let (Ok(rot), Ok(pos)) = (
        camera_rotation_transform.get_single(),
        camera_position_transform.get_single(),
    ) {
        let forward = rot.forward();
        let pos = pos.translation;
        if let Some(ray_hit) = spatial_query.cast_ray(
            pos,
            forward,
            MAX_INTERACTION_DISTANCE,
            false,
            SpatialQueryFilter::new().with_masks([RigidLayer::Ground]),
        ) {
            let face = {
                let mut to_return = None;
                if ray_hit.normal == Vec3::Y {
                    to_return = Some(Top);
                }
                if ray_hit.normal == Vec3::NEG_Y {
                    to_return = Some(Bottom);
                }
                if ray_hit.normal == Vec3::X {
                    to_return = Some(Right);
                }
                if ray_hit.normal == Vec3::NEG_X {
                    to_return = Some(Left);
                }
                if ray_hit.normal == Vec3::Z {
                    to_return = Some(Back);
                }
                if ray_hit.normal == Vec3::NEG_Z {
                    to_return = Some(Forward);
                }
                to_return
            };
            let impact_point = pos + ray_hit.time_of_impact * forward + SMALL_TRAVERSE * forward;
            // println!(
            //     "impact point: {}, pos: {}, forward: {}, time_of_impact: {},",
            //     impact_point, pos, forward, ray_hit.time_of_impact
            // );
            let (chunk_cords, block_index, _) =
                position_to_chunk_position(impact_point, CHUNK_DIMS);
            *target_block = TargetBlock {
                target_entity: ray_hit.entity,
                chunk_cords,
                block_index: one_d_cords(block_index, CHUNK_DIMS),
                face_hit: face,
                ray_direction: forward,
            };
        }
    }
}

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

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

fn cursor_grab(
    keys: Res<Input<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        if keys.just_pressed(KeyCode::Escape) {
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
            .init_resource::<TargetBlock>()
            .insert_resource(CurrentChunk([0, 0]))
            .add_systems(Startup, initial_grab_cursor)
            .add_systems(
                Update,
                setup_player.run_if(
                    not(any_with_component::<ComputeChunk>())
                        .and_then(not(any_with_component::<PlayerCamera>()))
                        .and_then(any_with_component::<Chunk>()),
                ),
            )
            .add_systems(
                Update,
                (
                    update_current_chunk,
                    player_look,
                    update_target_block,
                    cursor_grab,
                ),
            );
    }
}

fn update_current_chunk(
    mut current_chunk: ResMut<CurrentChunk>,
    player: Query<&Transform, With<PhysicalPlayer>>,
) {
    if let Ok(t) = player.get_single() {
        let tmp = position_to_chunk(t.translation, CHUNK_DIMS);
        if tmp != current_chunk.0 {
            current_chunk.0 = tmp;
        }
    }
}

impl Default for TargetBlock {
    fn default() -> Self {
        TargetBlock {
            target_entity: Entity::PLACEHOLDER,
            chunk_cords: [0, 0],
            block_index: 0,
            face_hit: None,
            ray_direction: Vec3::ONE,
        }
    }
}
