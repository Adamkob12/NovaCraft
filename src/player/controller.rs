// REFACTORED

pub mod movement;

pub use super::{PhysicalPlayer, PlayerGameMode, CAMERA_HEIGHT_OFFSET, FOV};
use bevy::prelude::*;
use bevy_xpbd_3d::{math::*, prelude::*};
pub use movement::*;

/// Speed of the player (speed of the player should be independant of framerate)
pub const SPEED: f32 = 500.0;
/// The maximum time (in seconds) that seperates the time of two clicks of the same key before it
/// counts as a "double click"
pub const DOUBLE_CLICK_MAX_SEP_TIME: f32 = 0.22;
/// The movement damping factor is scaler amount that will be multiplied against the velocity each frame.
pub const MOVEMENT_DAMPING_FACTOR: f32 = 0.72;
/// The velocity that a controller's subject will recieve at the moment of jumping.
pub const JUMP_IMPULSE: f32 = 8.0;
/// If the angle at the collision point between the controller's subject and the ground less than
/// this value ([`MAX_SLOPE_ANGLE`]) - the subject would be treated as [`Grounded`]. else not.
pub const MAX_SLOPE_ANGLE: f32 = PI * 2.0;
/// Drag in this case is the exponent of [`MovementDampingFactor`] while the controller's subject
/// is not grounded.
pub const DRAG: i32 = 10;

pub struct CharacterControllerPlugin;

/// An event sent for a movement input action.
#[derive(Event)]
pub enum MovementAction {
    Move(Vec3),
    Jump,
}

/// Marker component for flying
#[derive(Component, Debug)]
pub struct FlyMode;

/// Marker component for no clip
#[derive(Component, Debug)]
pub struct NoClipMode;

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

/// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;
/// The acceleration used for character movement.
#[derive(Component)]
pub struct Speed(Scalar);

/// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor(Scalar);

/// The strength of a jump.
#[derive(Component)]
pub struct JumpImpulse(Scalar);

/// The maximum angle a slope can have for a character controller
/// to be able to climb and jump. If the slope is steeper than this angle,
/// the character will slide down.
#[derive(Component)]
pub struct MaxSlopeAngle(Scalar);

/// A bundle that contains the components needed for a basic
/// kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    locked_axes: LockedAxes,
    movement: MovementBundle,
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    speed: Speed,
    dampning: MovementDampingFactor,
    jump_impulse: JumpImpulse,
    max_slope_angle: MaxSlopeAngle,
}

impl MovementBundle {
    pub const fn new(
        speed: Scalar,
        dampning: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        Self {
            speed: Speed(speed),
            dampning: MovementDampingFactor(dampning),
            jump_impulse: JumpImpulse(jump_impulse),
            max_slope_angle: MaxSlopeAngle(max_slope_angle),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(
            SPEED,
            MOVEMENT_DAMPING_FACTOR,
            JUMP_IMPULSE,
            MAX_SLOPE_ANGLE,
        )
    }
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider) -> Self {
        // Create shape caster as a slightly smaller version of collider
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.99, 10);

        Self {
            character_controller: CharacterController,
            rigid_body: RigidBody::Dynamic,
            collider,
            ground_caster: ShapeCaster::new(
                caster_shape,
                Vector::ZERO,
                Quaternion::default(),
                Vector::NEG_Y,
            )
            .with_query_filter(
                SpatialQueryFilter::new().with_masks([crate::player::RigidLayer::Ground]),
            )
            .with_max_time_of_impact(0.2),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            movement: MovementBundle::default(),
        }
    }

    pub fn with_movement(
        mut self,
        speed: Scalar,
        dampning: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(speed, dampning, jump_impulse, max_slope_angle);
        self
    }
}

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MovementAction>().add_systems(
            Update,
            (
                keyboard_input,
                update_grounded,
                apply_deferred,
                movement,
                apply_dampning,
            )
                .chain(),
        );
    }
}
