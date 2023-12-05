use super::*;

/// Grabs/ungrabs mouse cursor
pub(super) fn toggle_grab_cursor(window: &mut Window) {
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

/// Updates the current chunk the player is in
pub(super) fn update_current_chunk(
    mut current_chunk: ResMut<CurrentChunk>,
    player: Query<&Transform, With<PhysicalPlayer>>,
) {
    if let Ok(t) = player.get_single() {
        let tmp = point_to_chunk_cords(t.translation, CHUNK_DIMS);
        if tmp != current_chunk.0 {
            current_chunk.0 = tmp;
        }
    }
}

/// Grabs the cursor when game first starts
pub(super) fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

/// grab cursor when escape is pressed and ungrab when escape is pressed again
pub(super) fn cursor_grab(
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

/// Allow the player to cycle between game modes with the L key
pub(super) fn cycle_game_mode(keys: Res<Input<KeyCode>>, mut player: Query<&mut PlayerGameMode>) {
    if keys.just_pressed(KeyCode::L) {
        if let Ok(mut game_mode) = player.get_single_mut() {
            game_mode.cycle();
            println!("Game mode: {:?}", game_mode);
        }
    }
}

/// Listen for changes in game mode and update the player accordingly
pub(super) fn update_player_according_to_gamemode(
    mut player_query: Query<
        (Entity, &PlayerGameMode, Has<FlyMode>, Has<NoClipMode>),
        Changed<PlayerGameMode>,
    >,
    mut commands: Commands,
) {
    for (player_entity, gamemode, is_flying, is_noclip) in player_query.iter_mut() {
        if is_flying {
            if !gamemode.can_fly() {
                commands.entity(player_entity).remove::<FlyMode>();
            }
        } else {
            if gamemode.must_fly() {
                commands.entity(player_entity).insert(FlyMode);
            }
        }

        if is_noclip {
            if !gamemode.can_noclip() {
                commands.entity(player_entity).remove::<NoClipMode>();
            }
        } else {
            if gamemode.must_noclip() {
                commands.entity(player_entity).insert(NoClipMode);
            }
        }
    }
}

/// change the collision layers of the player in noclip mode, so that they don't collide with anything
pub(super) fn update_player_collision_layers(
    mut player_query: Query<(Has<NoClipMode>, &mut CollisionLayers)>,
    added_noclip_query: Query<Entity, Added<NoClipMode>>,
    mut removed_noclip: RemovedComponents<NoClipMode>,
) {
    for removed_noclip_entity in removed_noclip.read() {
        if let Ok((has_noclip_mode, mut collision_layers)) =
            player_query.get_mut(removed_noclip_entity)
        {
            if !has_noclip_mode {
                *collision_layers = build_player_collision_layers();
            }
        }
    }
    for added_noclip_entity in added_noclip_query.iter() {
        if let Ok((has_noclip_mode, mut collision_layers)) =
            player_query.get_mut(added_noclip_entity)
        {
            if has_noclip_mode {
                *collision_layers = build_spectator_collision_layers();
            }
        }
    }
}

/// Set the gravity of the player to 0 when in fly mode
pub(super) fn update_player_gravity(
    mut player_query: Query<(Has<FlyMode>, &mut GravityScale)>,
    added_fly_query: Query<Entity, Added<FlyMode>>,
    mut removed_fly: RemovedComponents<FlyMode>,
) {
    for removed_fly_entity in removed_fly.read() {
        if let Ok((has_fly_mode, mut gravity)) = player_query.get_mut(removed_fly_entity) {
            if !has_fly_mode {
                *gravity = PLAYER_GRAVITY_SCALE;
            }
        }
    }
    for added_fly_entity in added_fly_query.iter() {
        if let Ok((has_fly_mode, mut gravity)) = player_query.get_mut(added_fly_entity) {
            if has_fly_mode {
                *gravity = FLYMODE_GRAVITY_SCALE;
            }
        }
    }
}

/// Disable sprinting when velocity is too low.
/// If the velocity is under a certain threshold, set it to 0, and remove the sprinting component.
pub(super) fn nullify_velocity_when_velocity_is_too_low(
    mut player_query: Query<&mut LinearVelocity>,
) {
    for mut velocity in player_query.iter_mut() {
        if velocity.length() < SPRINT_THRESHOLD {
            **velocity = Vec3::ZERO;
        }
    }
}
