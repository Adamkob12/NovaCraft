mod block_highlight;
mod crosshair;
mod debug_mode;

pub(super) use crate::player::TargetBlock;
pub(super) use crate::prelude::*;
#[allow(unused_imports)]
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use block_highlight::*;
use crosshair::*;

use self::debug_mode::*;

pub struct VisualsPlugin;

#[derive(Component)]
pub struct FpsText;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        //
        // FPS
        //
        app.add_plugins((FrameTimeDiagnosticsPlugin::default(),));
        app.add_systems(Startup, (config_gizmos, setup_fps))
            .add_systems(PostUpdate, fps_update);
        //
        // BLOCK HIGHLIGHT
        //
        app.add_systems(PostUpdate, highlight_target_block);
        //
        // CROSSHAIR
        //
        app.init_resource::<CrossHairConfig>();
        app.add_systems(
            Update,
            setup_crosshair.run_if(
                resource_changed::<CrossHairConfig>().or_else(resource_added::<CrossHairConfig>()),
            ),
        );
        //
        // DEBUG MODE
        //
        app.init_resource::<DebugModeSettings>();
        app.add_state::<DebugMode>();
        app.add_systems(Startup, setup_debug_mode);
        app.add_systems(OnEnter::<DebugMode>(DebugMode::On), show_debug_mode);
        app.add_systems(OnEnter::<DebugMode>(DebugMode::Off), hide_debug_mode);
        app.add_systems(
            PreUpdate,
            (
                toggle_debug_mode,
                update_debug_ui.run_if(in_state::<DebugMode>(DebugMode::On)),
            ),
        );
    }
}

fn setup_fps(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server.load("fonts/main_font.ttf");
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([TextSection::from_style(TextStyle {
            font_size: 50.0,
            font: font_handle,
            color: Color::GOLD,
            // If no font is specified, the default font (a minimal subset of FiraMono) will be used.
            ..default()
        })])
        .with_style(Style {
            left: Val::Percent(95.0),
            top: Val::Percent(95.0),
            ..Default::default()
        }),
        FpsText,
    ));
}

fn fps_update(diagnostics: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[0].value = format!("{value:.0}");
            }
        }
    }
}
