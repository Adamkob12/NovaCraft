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
