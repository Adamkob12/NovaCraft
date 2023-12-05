use super::*;
use super::{PlayerGameMode, CAMERA_HEIGHT_OFFSET, FOV};
use crate::player::*;
use bevy::{ecs::query::Has, prelude::*, utils::HashMap, utils::Instant};
use bevy_xpbd_3d::{math::*, prelude::*};

/// Sends [`MovementAction`] events based on keyboard input.
pub(super) fn keyboard_input(
    mut commands: Commands,
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<Input<KeyCode>>,
    camera_query: Query<&Transform, With<PlayerCamera>>,
    gamemode_query: Query<(Entity, &PlayerGameMode, Has<FlyMode>)>,
    mut press_history: ResMut<LastPressedKeys>,
) {
    let up = keyboard_input.any_pressed([KeyCode::W, KeyCode::Up]);
    let down = keyboard_input.any_pressed([KeyCode::S, KeyCode::Down]);
    let left = keyboard_input.any_pressed([KeyCode::A, KeyCode::Left]);
    let right = keyboard_input.any_pressed([KeyCode::D, KeyCode::Right]);

    let mut direction = Vec3::ZERO;
    let mut pressed_keys = vec![];
    if let Ok(tran) = camera_query.get_single() {
        let horizontal = (right as i8 - left as i8) as f32 * tran.right();
        let vertical = (up as i8 - down as i8) as f32 * tran.forward();
        direction = horizontal + vertical;
        direction.y = 0.0;
        direction = direction.normalize_or_zero();
    }
    if direction != Vec3::ZERO {
        movement_event_writer.send(MovementAction::Move(direction));
    } else {
        movement_event_writer.send(MovementAction::Nop);
    }

    if keyboard_input.pressed(KeyCode::Space) {
        movement_event_writer.send(MovementAction::Jump);
        pressed_keys.push(KeyCode::Space);
    }
    if keyboard_input.pressed(KeyCode::ControlLeft) {
        pressed_keys.push(KeyCode::ControlLeft);
    }
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        pressed_keys.push(KeyCode::ShiftLeft);
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok((player_entity, player_game_mode, is_flying)) = gamemode_query.get_single() {
            if let Some(last_press) = press_history.map.get(&KeyCode::Space) {
                if last_press.elapsed().as_secs_f32() <= DOUBLE_CLICK_MAX_SEP_TIME {
                    if is_flying && !player_game_mode.must_fly() {
                        commands.entity(player_entity).remove::<FlyMode>();
                    }
                    if !is_flying && player_game_mode.can_fly() {
                        commands.entity(player_entity).insert(FlyMode);
                    }
                }
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::ControlLeft) {
        movement_event_writer.send(MovementAction::CrouchStart);
    }
    if keyboard_input.just_released(KeyCode::ControlLeft) {
        movement_event_writer.send(MovementAction::CrouchStop);
    }
    if keyboard_input.just_pressed(KeyCode::ShiftLeft) {
        movement_event_writer.send(MovementAction::SprintStart);
    }
    if keyboard_input.just_released(KeyCode::ShiftLeft) {
        movement_event_writer.send(MovementAction::SprintStop);
    }
    if up {
        pressed_keys.push(KeyCode::W);
    }
    if down {
        pressed_keys.push(KeyCode::S);
    }
    if left {
        pressed_keys.push(KeyCode::A);
    }
    if right {
        pressed_keys.push(KeyCode::D);
    }
    for key in pressed_keys {
        press_history.map.insert(key, Instant::now());
    }
}

/// Updates the [`Grounded`] status for character controllers.
pub(super) fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>),
        (With<CharacterController>, Without<FlyMode>),
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
    mut commands: Commands,
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controllers: Query<(
        Entity,
        &Speed,
        &JumpImpulse,
        &mut LinearVelocity,
        Has<Grounded>,
        Has<FlyMode>,
        Has<Crouched>,
        Has<Sprinting>,
    )>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_seconds_f64().adjust_precision();

    for (
        player_entity,
        speed,
        jump_impulse,
        mut linear_velocity,
        is_grounded,
        is_flying,
        is_crouched,
        is_sprinting,
    ) in &mut controllers
    {
        if is_crouched && is_flying {
            linear_velocity.y = -jump_impulse.0 * 1.4;
        }
        for event in movement_event_reader.read() {
            match event {
                MovementAction::Nop if is_sprinting => {
                    commands.entity(player_entity).remove::<Sprinting>();
                }
                MovementAction::Move(direction) => {
                    // linear_velocity.x += direction.x * speed.0 * delta_time;
                    // linear_velocity.z += direction.z * speed.0 * delta_time;
                    let tmp = *direction * speed.0 * delta_time;
                    linear_velocity.x = tmp.x;
                    linear_velocity.z = tmp.z;
                }
                MovementAction::Jump => {
                    if is_grounded || is_flying {
                        linear_velocity.y = jump_impulse.0;
                    }
                }
                MovementAction::CrouchStart | MovementAction::CrouchStop => {
                    if is_crouched {
                        commands.entity(player_entity).remove::<Crouched>();
                    } else {
                        commands.entity(player_entity).insert(Crouched);
                    }
                }
                MovementAction::SprintStart => {
                    if is_sprinting {
                        commands.entity(player_entity).remove::<Sprinting>();
                    } else {
                        commands.entity(player_entity).insert(Sprinting);
                    }
                }
                MovementAction::SprintStop => {}
                MovementAction::Nop => {}
            }
        }
    }
}

/// Apply movement dampning, the player will keep moving as long as he is pressing a button, but
/// the moment he stops, the movement dampning will slowly push the velocity towards 0.
/// This provides a friction-like effect.
pub(super) fn apply_dampning(
    mut movement: Query<(
        &mut LinearVelocity,
        &mut Speed,
        Has<Grounded>,
        Has<FlyMode>,
        Has<Crouched>,
        Has<Sprinting>,
    )>,
) {
    for (mut linear_velocity, mut speed, grounded, is_flying, is_crouched, is_sprinting) in
        movement.iter_mut()
    {
        let mut total_multiplier = 1.0;
        if is_crouched {
            total_multiplier *= CROUCH_SPEED_SCALER;
        }
        if is_sprinting {
            total_multiplier *= SPRINT_SPEED_SCALER;
        }
        if is_flying {
            total_multiplier *= FLYING_SPEED_SCALER;
        }

        speed.0 = total_multiplier * SPEED;

        let factor = if grounded && !is_flying { 1.0 } else { 0.96 };

        if is_flying {
            **linear_velocity *= FLYING_DAMPING_FACTOR;
        } else {
            linear_velocity.x *= MOVEMENT_DAMPING_FACTOR.powf(factor);
            linear_velocity.z *= MOVEMENT_DAMPING_FACTOR.powf(factor);
        }
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

/// Handle crouch and sprint. Fov and camera height is adjusted accordingly.
pub(super) fn handle_crouch_sprint(
    mut camera_query: Query<(&mut Projection, &mut Transform), With<PlayerCamera>>,
    player_query: Query<(Has<Crouched>, Has<Sprinting>, &Children), With<PhysicalPlayer>>,
) {
    for (is_cruched, is_sprinting, player_children) in player_query.iter() {
        for child in player_children {
            if let Ok((mut projection, mut transform)) = camera_query.get_mut(*child) {
                if let Projection::Perspective(ref mut perspective) = *projection {
                    if is_cruched {
                        transform.translation.y = CROUCH_CAMERA_HEIGHT_OFFSET;
                        perspective.fov = CROUCH_FOV;
                    } else {
                        transform.translation.y = CAMERA_HEIGHT_OFFSET;
                        perspective.fov = FOV;
                    }
                    if is_sprinting {
                        perspective.fov = SPRINT_FOV;
                    } else {
                        perspective.fov = FOV;
                    }
                }
            }
        }
    }
}
