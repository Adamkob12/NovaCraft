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

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//                                         CONSTANTS
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Velocity value threshold for the player to be sprinting
pub const SPRINT_THRESHOLD: f32 = 0.1;
/// Speed scaler when sprinting
pub const SPRINT_SPEED_SCALER: f32 = 1.42;
/// Speed scaler when coruching
pub const CROUCH_SPEED_SCALER: f32 = 0.56;
/// Speed of the player (speed of the player should be independant of framerate)
pub const SPEED: f32 = 540.0;
/// The maximum time (in seconds) that seperates the time of two clicks of the same key before it
/// counts as a "double click"
pub const DOUBLE_CLICK_MAX_SEP_TIME: f32 = 0.15;
/// The movement damping factor is scaler amount that will be multiplied against the velocity each frame.
pub const MOVEMENT_DAMPING_FACTOR: f32 = 0.60;
/// The flying damping factor is scaler amount that will be multiplied against the velocity each frame when flying.
pub const FLYING_DAMPING_FACTOR: f32 = 0.90;
/// The velocity that a controller's subject will recieve at the moment of jumping.
pub const JUMP_IMPULSE: f32 = 9.0;
/// If the angle at the collision point between the controller's subject and the ground less than
/// this value ([`MAX_SLOPE_ANGLE`]) - the subject would be treated as [`Grounded`]. else not.
pub const MAX_SLOPE_ANGLE: f32 = PI * 2.0;
/// Drag in this case is the exponent of [`MovementDampingFactor`] while the controller's subject
/// is not grounded.
pub const DRAG: i32 = 10;
/// Scaler of the player's speed when flying
pub const FLYING_SPEED_SCALER: f32 = 1.5;
/// Default player speed when flying
pub const FLYING_SPEED: f32 = SPEED * FLYING_SPEED_SCALER;
/// Default SSAO quality level
pub const SSAO_QUALITY_LEVEL: ScreenSpaceAmbientOcclusionQualityLevel =
    ScreenSpaceAmbientOcclusionQualityLevel::High;
/// Default fog color
pub const FOG_COLOR: Color = Color::rgb(0.65, 0.95, 1.0);
/// Default fog falloff
pub const FOG_FALLOFF: FogFalloff = FogFalloff::Linear {
    start: ((RENDER_DISTANCE - 2) * WIDTH as i32) as f32,
    end: ((RENDER_DISTANCE + 1) * WIDTH as i32) as f32,
};
/// Starting position of the player
pub const STARTING_POS: [f32; 3] = [0.0, HEIGHT as f32 + 5.0, 0.0];
/// Starting chunk of the player
pub const STARTING_CHUNK: ChunkCords = ChunkCords::new(
    STARTING_POS[0] as i32 / WIDTH as i32,
    STARTING_POS[2] as i32 / LENGTH as i32,
);
/// We don't want the camera to be exactly where the player's collider is, because that's the
/// center of the collider. This constant offsets the camera's position to match eye level.
pub const CAMERA_HEIGHT_OFFSET: f32 = 0.45;
/// Camera offset while crouched
pub const CROUCH_CAMERA_HEIGHT_OFFSET: f32 = CAMERA_HEIGHT_OFFSET * 0.35;
/// Camera starting pos
pub const CAMERA_STARTING_POS: [f32; 3] = [0.0, 0.0 + CAMERA_HEIGHT_OFFSET, 0.0];
/// The "reach" of the player, what is the largest distance from the player that a block can be and
/// the player can break it / interact with it.
pub const MAX_INTERACTION_DISTANCE: f32 = 6.0;
/// Epsilon to offset a point by a little bit
pub const SMALL_TRAVERSE: f32 = 0.001;
/// Default Field of view
pub const FOV: f32 = PI / 3.0;
/// FOV while croching
pub const CROUCH_FOV: f32 = FOV * 0.97;
/// FOV while sprinting
pub const SPRINT_FOV: f32 = FOV * 1.05;
/// Distance which after the camera won't render anything
pub const FAR: f32 = (RENDER_DISTANCE + 3) as f32 * WIDTH as f32;
/// Default player collider height
pub const PLAYER_COLLIDER_HEIGHT: f32 = 1.15;
/// Default player collider radius
pub const PLAYER_COLLIDER_RADIUS: f32 = 0.42;
/// Default player fricition
pub const PLAYER_FRICTION: Friction = Friction::ZERO;
/// Default player restitution
pub const PLAYER_RESTITUTION: Restitution = Restitution::ZERO;
/// Default player gravity scale
pub const PLAYER_GRAVITY_SCALE: GravityScale = GravityScale(3.5);
/// Flymode gravity scale
pub const FLYMODE_GRAVITY_SCALE: GravityScale = GravityScale(0.0);
/// Default player collision groups
pub const PLAYER_GROUPS: &[RigidLayer] = &[RigidLayer::Player];
/// Default player collision masks
pub const PLAYER_MASKS: &[RigidLayer] = &[RigidLayer::Ground];
/// Default collision groups in spectator mode
pub const SPECTATOR_GROUPS: &[RigidLayer] = &[RigidLayer::Spectator];
/// Default collision masks in spectator mode
pub const SPECTATOR_MASKS: &[RigidLayer] = &[];
/// Build player collider
pub fn build_player_collider() -> Collider {
    Collider::capsule(PLAYER_COLLIDER_HEIGHT, PLAYER_COLLIDER_RADIUS)
}
/// Build player fricition
pub fn build_player_friction() -> Friction {
    PLAYER_FRICTION.with_combine_rule(CoefficientCombine::Min)
}
/// Build player restitution
pub fn build_player_restitution() -> Restitution {
    PLAYER_RESTITUTION.with_combine_rule(CoefficientCombine::Min)
}
/// Build player collision layers
pub fn build_player_collision_layers() -> CollisionLayers {
    CollisionLayers::new(PLAYER_GROUPS, PLAYER_MASKS)
}
/// Build spectator collision layers
pub fn build_spectator_collision_layers() -> CollisionLayers {
    CollisionLayers::new(SPECTATOR_GROUPS, SPECTATOR_MASKS)
}
/// Default TAA
#[allow(non_snake_case)]
fn TAA() -> TemporalAntiAliasBundle {
    TemporalAntiAliasBundle::default()
}
/// Default SSAO as function
#[allow(non_snake_case)]
fn SSAO() -> ScreenSpaceAmbientOcclusionBundle {
    ScreenSpaceAmbientOcclusionBundle {
        settings: ScreenSpaceAmbientOcclusionSettings {
            quality_level: SSAO_QUALITY_LEVEL,
        },
        ..Default::default()
    }
}
/// This component marks the entity of the player's camera.
#[derive(Component)]
pub struct PlayerCamera;

/// This component marks the entity of the player's physical entity, all of its physics related
/// components are in the same entity. ([`Collider`], [`CollisionLayers`], [`GravityScale`], etc.)
#[derive(Component)]
pub struct PhysicalPlayer;

/// The gamemode of a player, each gamemode allows the player different things, gives him different
/// options. Foe example, [`Spectator`](PlayerGameMode::Spectator) allows him to fly through walls.
/// Mostly parralel to Minecraft's gamemodes
#[derive(Component, Debug)]
pub enum PlayerGameMode {
    Creative,
    Survival,
    Adventure,
    Spectator,
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

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Resource, Default)]
pub struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
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
    Spectator,
    FallingBlock,
    Ground,
    GroundNonCollidable,
    GroundNonBreakable,
}

impl PlayerGameMode {
    pub fn can_fly(&self) -> bool {
        match self {
            Self::Creative | Self::Spectator => true,
            Self::Survival | Self::Adventure => false,
        }
    }

    pub fn must_fly(&self) -> bool {
        match self {
            Self::Spectator => true,
            Self::Creative | Self::Survival | Self::Adventure => false,
        }
    }

    pub fn can_noclip(&self) -> bool {
        match self {
            Self::Spectator => true,
            Self::Creative => false,
            Self::Survival | Self::Adventure => false,
        }
    }

    pub fn must_noclip(&self) -> bool {
        match self {
            Self::Spectator => true,
            Self::Creative => false,
            Self::Survival | Self::Adventure => false,
        }
    }

    pub fn can_break_blocks(&self) -> bool {
        match self {
            Self::Creative => true,
            Self::Spectator => false,
            Self::Adventure => false,
            Self::Survival => true,
        }
    }

    pub fn cycle(&mut self) {
        match self {
            Self::Creative => *self = Self::Spectator,
            Self::Spectator => *self = Self::Adventure,
            Self::Survival => *self = Self::Creative,
            Self::Adventure => *self = Self::Survival,
        }
    }

    pub fn set(&mut self, mode: Self) {
        *self = mode;
    }
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

/// Physical player bundle
#[derive(Bundle)]
pub struct PhysicalPlayerBundle {
    pub player: PhysicalPlayer,
    pub spatial: SpatialBundle,
    pub controller: CharacterControllerBundle,
    pub friction: Friction,
    pub restitution: Restitution,
    pub gravity_scale: GravityScale,
    pub collision_layers: CollisionLayers,
    pub game_mode: PlayerGameMode,
}

impl Default for PhysicalPlayerBundle {
    fn default() -> Self {
        Self {
            player: PhysicalPlayer,
            spatial: SpatialBundle {
                transform: Transform::from_translation(STARTING_POS.into()),
                ..Default::default()
            },
            controller: CharacterControllerBundle::new(Collider::capsule(
                PLAYER_COLLIDER_HEIGHT,
                PLAYER_COLLIDER_RADIUS,
            )),
            friction: PLAYER_FRICTION.with_combine_rule(CoefficientCombine::Min),
            restitution: PLAYER_RESTITUTION.with_combine_rule(CoefficientCombine::Min),
            gravity_scale: PLAYER_GRAVITY_SCALE,
            collision_layers: CollisionLayers::new(PLAYER_GROUPS, PLAYER_MASKS),
            game_mode: PlayerGameMode::Creative,
        }
    }
}

/// Camera player bundle
#[derive(Bundle)]
pub struct PlayerCameraBundle {
    pub camera: PlayerCamera,
    pub camera_bundle: Camera3dBundle,
    pub atmosphere: AtmosphereCamera,
    pub fog: FogSettings,
}

impl Default for PlayerCameraBundle {
    fn default() -> Self {
        Self {
            camera: PlayerCamera,
            camera_bundle: Camera3dBundle {
                transform: Transform::from_translation(CAMERA_STARTING_POS.into()),
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: FOV,
                    far: FAR,
                    ..Default::default()
                }),
                ..Default::default()
            },
            atmosphere: AtmosphereCamera::default(),

            fog: FogSettings {
                color: FOG_COLOR,
                falloff: FOG_FALLOFF,
                ..Default::default()
            },
        }
    }
}

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(mut commands: Commands) {
    let player_entity = commands.spawn(PhysicalPlayerBundle::default()).id();
    let camera_entity = commands
        .spawn(PlayerCameraBundle::default())
        .insert(TAA())
        .insert(SSAO())
        .id();
    commands
        .entity(player_entity)
        .push_children(&[camera_entity]);
}

/// Updatees the target block, the block that the player is currently looking at
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
            .insert_resource(CurrentChunk(STARTING_CHUNK.into()))
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
                    nullify_velocity_when_velocity_is_too_low,
                    cycle_game_mode,
                    update_player_according_to_gamemode,
                    update_player_gravity,
                    update_player_collision_layers,
                    (player_look, update_target_block).chain(),
                    cursor_grab,
                ),
            );
    }
}
