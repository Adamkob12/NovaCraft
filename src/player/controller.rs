use crate::player::PlayerCamera;

use bevy::{ecs::query::Has, prelude::*};
use bevy_xpbd_3d::{math::*, prelude::*};

use super::{CAMERA_HEIGHT_OFFSET, FOV};

pub const SPEED: f32 = 420.0;

pub struct CharacterControllerPlugin;

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

/// An event sent for a movement input action.
#[derive(Event)]
pub enum MovementAction {
    Move(Vec3),
    Jump,
}

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
        Self::new(SPEED, 0.8, 6.5, PI * 0.2)
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

/// Sends [`MovementAction`] events based on keyboard input.
fn keyboard_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<(&mut Projection, &mut Transform), With<PlayerCamera>>,
    mut speed_query: Query<&mut Speed>,
) {
    let up = keyboard_input.any_pressed([KeyCode::W, KeyCode::Up]);
    let down = keyboard_input.any_pressed([KeyCode::S, KeyCode::Down]);
    let left = keyboard_input.any_pressed([KeyCode::A, KeyCode::Left]);
    let right = keyboard_input.any_pressed([KeyCode::D, KeyCode::Right]);

    let mut direction = Vec3::ZERO;
    if let Ok((_, tran)) = camera_query.get_single() {
        let horizontal = (right as i8 - left as i8) as f32 * tran.right();
        let vertical = (up as i8 - down as i8) as f32 * tran.forward();
        direction = horizontal + vertical;
        direction.y = 0.0;
        direction = direction.normalize_or_zero();
    }

    if direction != Vec3::ZERO {
        movement_event_writer.send(MovementAction::Move(direction));
    }

    if keyboard_input.pressed(KeyCode::Space) {
        movement_event_writer.send(MovementAction::Jump);
    }

    if keyboard_input.just_pressed(KeyCode::ShiftLeft) {
        if let (Ok((mut projection, _)), Ok(mut speed)) =
            (camera_query.get_single_mut(), speed_query.get_single_mut())
        {
            if let Projection::Perspective(ref mut perspective) = *projection {
                perspective.fov = FOV * 1.06;
            }
            speed.0 = SPEED * 1.5;
            return;
        }
    } else if keyboard_input.just_released(KeyCode::ShiftLeft) {
        if let (Ok((mut projection, _)), Ok(mut speed)) =
            (camera_query.get_single_mut(), speed_query.get_single_mut())
        {
            if let Projection::Perspective(ref mut perspective) = *projection {
                perspective.fov = FOV;
            }
            speed.0 = SPEED;
        }
    } else if keyboard_input.just_pressed(KeyCode::ControlLeft) {
        if let (Ok((mut projection, mut tran)), Ok(mut speed)) =
            (camera_query.get_single_mut(), speed_query.get_single_mut())
        {
            if let Projection::Perspective(ref mut perspective) = *projection {
                perspective.fov = FOV * 0.97;
            }
            speed.0 = SPEED / 1.5;
            tran.translation.y = CAMERA_HEIGHT_OFFSET / 3.0;
        }
    } else if keyboard_input.just_released(KeyCode::ControlLeft) {
        if let (Ok((mut projection, mut tran)), Ok(mut speed)) =
            (camera_query.get_single_mut(), speed_query.get_single_mut())
        {
            if let Projection::Perspective(ref mut perspective) = *projection {
                perspective.fov = FOV;
            }
            speed.0 = SPEED;
            tran.translation.y = CAMERA_HEIGHT_OFFSET;
        }
    }
}

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>),
        With<CharacterController>,
    >,
) {
    for (entity, hits, rotation, max_slope_angle) in &mut query {
        // The character is grounded if the shape caster has a hit with a normal
        // that isn't too steep.
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                rotation.rotate(-hit.normal2).angle_between(Vector::Y).abs() <= angle.0
            } else {
                true
            }
        });

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

/// Responds to [`MovementAction`] events and moves character controllers accordingly.
fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controllers: Query<(&Speed, &JumpImpulse, &mut LinearVelocity, Has<Grounded>)>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_seconds_f64().adjust_precision();

    for event in movement_event_reader.read() {
        for (speed, jump_impulse, mut linear_velocity, is_grounded) in &mut controllers {
            match event {
                MovementAction::Move(direction) => {
                    // linear_velocity.x += direction.x * speed.0 * delta_time;
                    // linear_velocity.z += direction.z * speed.0 * delta_time;
                    let tmp = *direction * speed.0 * delta_time;
                    linear_velocity.x = tmp.x;
                    linear_velocity.z = tmp.z;
                }
                MovementAction::Jump => {
                    if is_grounded {
                        linear_velocity.y = jump_impulse.0;
                    }
                }
            }
        }
    }
}

fn apply_dampning(
    mut movement: Query<(&mut LinearVelocity, &MovementDampingFactor, Has<Grounded>)>,
) {
    for (mut linear_velocity, dampning, grounded) in movement.iter_mut() {
        let factor = if grounded { 1.0 } else { 1.4 };
        linear_velocity.x *= dampning.0.powf(factor);
        linear_velocity.z *= dampning.0.powf(factor);
    }
}
