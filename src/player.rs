pub mod controller;
mod misc_systems;

use bevy_atmosphere::prelude::*;
use misc_systems::*;
use std::f32::consts::PI;

use crate::chunk::{
    ChunkCords, ComputeChunk, CurrentChunk, CHUNK_DIMS, HEIGHT, LENGTH, RENDER_DISTANCE, WIDTH,
};
use crate::{prelude::*, AssetLoadingState};
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

/// We don't want the camera to be exactly where the player's collider is, because that's the
/// center of the collider. This constant offsets the camera's position to match eye level.
pub const CAMERA_HEIGHT_OFFSET: f32 = 0.45;
/// The "reach" of the player, what is the largest distance from the player that a block can be and
/// the player can break it / interact with it.
pub const MAX_INTERACTION_DISTANCE: f32 = 6.0;
/// Epsilon to offset a point by a little bit
pub const SMALL_TRAVERSE: f32 = 0.001;
/// Default Field of view
pub const FOV: f32 = PI / 3.0;

/// This component marks the entity of the player's camera.
#[derive(Component)]
pub struct PlayerCamera;

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Resource, Default)]
pub struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

/// This component marks the entity of the player's physical entity, all of its physics related
/// components are in the same entity. ([`Collider`], [`CollisionLayers`], [`GravityScale`], etc.)
#[derive(Component)]
pub struct PhysicalPlayer;

/// The gamemode of a player, each gamemode allows the player different things, gives him different
/// options. Foe example, [`Spectator`](PlayerGameMode::Spectator) allows him to fly through walls.
/// Mostly parralel to Minecraft's gamemodes
#[derive(Component)]
pub enum PlayerGameMode {
    Creative,
    Spectator,
}

impl PlayerGameMode {
    pub fn can_fly(&self) -> bool {
        match self {
            Self::Creative | Self::Spectator => true,
        }
    }
}

/// This resource represents the block that the player is currently looking at ("targeting")
/// It's updated every frame using a [`RayCaster`]
#[derive(Resource)]
pub struct TargetBlock {
    pub ignore_flag: bool,
    pub target_entity: Entity,
    pub chunk_cords: ChunkCords,
    pub block_pos: BlockPos,
    pub face_hit: Option<Face>,
    pub ray_direction: Vec3,
}

/// Mouse sensitivity and movement speed
#[derive(Resource)]
pub struct MovementSettings {
    pub sensitivity: f32,
}

/// An enum of all the possible physics layers in the world. A [`PhysicsLayer`] is a physical
/// attribute that allows [`Physical Queries made by the physics engine`](`SpatialQuery`) to filter
/// out specific colliders. For example, the players shouldn't collide with grass, so we might
/// filter out the grasses [`PhysicsLayer`] when setting up the [`player's collider`](Collider), but we still want the
/// player to be able to break grass, so we will include its [`PhysicsLayer`] in the [`RayCaster`].
#[repr(C)]
#[derive(PhysicsLayer, Copy, Clone)]
pub enum RigidLayer {
    Player,
    FallingBlock,
    Ground,
    GroundNonCollidable,
    GroundNonBreakable,
}

impl Default for TargetBlock {
    fn default() -> Self {
        TargetBlock {
            ignore_flag: true,
            target_entity: Entity::PLACEHOLDER,
            chunk_cords: [0, 0].into(),
            block_pos: [0, 0, 0].into(),
            face_hit: None,
            ray_direction: Vec3::ONE,
        }
    }
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.000037,
        }
    }
}

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(mut commands: Commands) {
    let player_entity = commands
        .spawn(PhysicalPlayer)
        .insert(SpatialBundle {
            transform: Transform::from_xyz(0.0, HEIGHT as f32 + 5.0, 0.0),
            ..Default::default()
        })
        .insert(CharacterControllerBundle::new(Collider::capsule(
            1.15, 0.42,
        )))
        .insert(Friction::ZERO.with_combine_rule(CoefficientCombine::Min))
        .insert(Restitution::ZERO.with_combine_rule(CoefficientCombine::Min))
        .insert(GravityScale(2.4))
        .insert(CollisionLayers::new(
            [RigidLayer::Player],
            [RigidLayer::Ground],
        ))
        .id();
    let camera_entity = commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, CAMERA_HEIGHT_OFFSET, 0.0),
            projection: Projection::Perspective(PerspectiveProjection {
                fov: FOV,
                far: (RENDER_DISTANCE + 3) as f32 * WIDTH.max(LENGTH) as f32,
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert(PlayerCamera)
        .insert(FlyMode::off())
        .insert(PlayerGameMode::Creative)
        .insert(AtmosphereCamera::default())
        .insert(TemporalAntiAliasBundle::default())
        .insert(ScreenSpaceAmbientOcclusionBundle {
            settings: ScreenSpaceAmbientOcclusionSettings {
                quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
            },
            ..Default::default()
        })
        .insert(FogSettings {
            color: Color::rgb(0.65, 0.95, 1.0),
            falloff: FogFalloff::Linear {
                start: ((RENDER_DISTANCE - 2) * WIDTH as i32) as f32,
                end: ((RENDER_DISTANCE + 1) * WIDTH as i32) as f32,
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
        let pos = pos.translation + rot.translation;
        if let Some(ray_hit) = spatial_query.cast_ray(
            pos,
            forward,
            MAX_INTERACTION_DISTANCE,
            false,
            SpatialQueryFilter::new()
                .with_masks([RigidLayer::Ground, RigidLayer::GroundNonCollidable]),
        ) {
            let face = {
                let mut to_return = None;
                if ray_hit.normal == Vec3::Y {
                    to_return = Some(Face::Top);
                }
                if ray_hit.normal == Vec3::NEG_Y {
                    to_return = Some(Face::Bottom);
                }
                if ray_hit.normal == Vec3::X {
                    to_return = Some(Face::Right);
                }
                if ray_hit.normal == Vec3::NEG_X {
                    to_return = Some(Face::Left);
                }
                if ray_hit.normal == Vec3::Z {
                    to_return = Some(Face::Back);
                }
                if ray_hit.normal == Vec3::NEG_Z {
                    to_return = Some(Face::Forward);
                }
                to_return
            };
            let impact_point = pos + ray_hit.time_of_impact * forward + SMALL_TRAVERSE * forward;
            let global_pos = point_to_global_block_pos(impact_point, CHUNK_DIMS);
            *target_block = TargetBlock {
                ignore_flag: false,
                target_entity: ray_hit.entity,
                chunk_cords: global_pos.chunk_cords,
                block_pos: global_pos.pos,
                face_hit: face,
                ray_direction: forward,
            };
        } else {
            target_block.ignore_flag = true;
        }
    }
}
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TemporalAntiAliasPlugin,))
            .init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .init_resource::<TargetBlock>()
            .init_resource::<LastPressedKeys>()
            .insert_resource(CurrentChunk([0, 0].into()))
            .add_systems(Startup, initial_grab_cursor)
            .add_systems(
                Update,
                setup_player.run_if(
                    not(any_with_component::<ComputeChunk>())
                        .and_then(not(any_with_component::<PlayerCamera>()))
                        .and_then(any_with_component::<Collider>())
                        .and_then(in_state::<AssetLoadingState>(AssetLoadingState::Loaded)),
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    update_current_chunk,
                    (player_look, update_target_block).chain(),
                    cursor_grab,
                ),
            );
    }
}

//
