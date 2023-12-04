// REFACTORED

use super::*;
use super::{PhysicalPlayer, PlayerGameMode, CAMERA_HEIGHT_OFFSET, FOV};
use crate::player::*;
use bevy::{ecs::query::Has, prelude::*, utils::HashMap, utils::Instant};
use bevy_xpbd_3d::{math::*, prelude::*};

/// Sends [`MovementAction`] events based on keyboard input.
pub(super) fn keyboard_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<(&mut Projection, &mut Transform), With<PlayerCamera>>,
    mut speed_query: Query<&mut Speed>,
    mut physical_player: Query<&mut FlyMode, With<PhysicalPlayer>>,
    player_game_mode: Query<&PlayerGameMode>,
    mut press_history: ResMut<LastPressedKeys>,
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
        // Handle double-click on space bar for fly mode
        if player_game_mode.get_single().unwrap().can_fly() {
            if let Some(last_press) = press_history.map.get(&KeyCode::Space) {
                if last_press.elapsed().as_secs_f32() <= DOUBLE_CLICK_MAX_SEP_TIME {
                    physical_player
                        .iter_mut()
                        .for_each(|mut flymode| flymode.toggle());
                }
                press_history.map.insert(KeyCode::Space, Instant::now());
            }
        }
    }

    // Sprint, Crouch
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
pub(super) fn update_grounded(
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
pub(super) fn movement(
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

/// Apply movement dampning, the player will keep moving as long as he is pressing a button, but
/// the moment he stops, the movement dampning will slowly push the velocity towards 0.
/// This provides a friction-like effect.
pub(super) fn apply_dampning(
    mut movement: Query<(&mut LinearVelocity, &MovementDampingFactor, Has<Grounded>)>,
) {
    for (mut linear_velocity, dampning, grounded) in movement.iter_mut() {
        let factor = if grounded { 1.0 } else { 1.4 };
        linear_velocity.x *= dampning.0.powf(factor);
        linear_velocity.z *= dampning.0.powf(factor);
    }
}

/// Handles looking around if cursor is locked
pub fn player_look(
    settings: Res<MovementSettings>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, With<PlayerCamera>>,
) {
    if let Ok(window) = primary_window.get_single() {
        for mut transform in query.iter_mut() {
            for ev in state.reader_motion.read(&motion) {
                let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                        let window_scale = window.height().min(window.width());
                        pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                        yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                    }
                }
                pitch = pitch.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

/// Resource to keep track how long ago each key was pressed. This is useful for checking for
/// double click for example, or other similar features that require input history.
#[derive(Resource, Default)]
pub struct LastPressedKeys {
    pub map: HashMap<KeyCode, Instant>,
}
