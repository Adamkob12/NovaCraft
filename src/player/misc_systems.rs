use super::*;

// Keeps track of the blocks surrounding the player for physics
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
