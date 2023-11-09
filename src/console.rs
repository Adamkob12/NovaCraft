mod commands;
pub(super) mod global_parameter;

use crate::{
    chunk::{RenderSettings, DEFAULT_PBS},
    prelude::*,
};
use bevy_console::*;
use clap::{builder::OsStr, Parser};
use commands::*;
use std::cmp;

pub struct GlobalConsolePlugin;

impl Plugin for GlobalConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConsolePlugin);
        app.insert_resource(ConsoleConfiguration {
            keys: vec![ToggleConsoleKey::KeyCode(KeyCode::Slash)],
            ..Default::default()
        });
        app.add_systems(PreUpdate, pause_game);
        app.add_console_command::<SetCommand, _>(set_command);
    }
}

fn pause_game(
    console_config: Res<ConsoleConfiguration>,
    mut time: ResMut<Time<Virtual>>,
    keys: Res<Input<KeyCode>>,
) {
    for key in console_config.keys.iter() {
        if let ToggleConsoleKey::KeyCode(key) = key {
            if keys.just_pressed(*key) {
                if time.is_paused() {
                    time.unpause();
                } else {
                    time.pause();
                }
            }
        }
    }
}
