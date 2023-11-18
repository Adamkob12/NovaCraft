mod block_highlight;
mod crosshair;

pub(super) use crate::player::TargetBlock;
pub(super) use crate::prelude::*;
#[allow(unused_imports)]
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use block_highlight::*;
use crosshair::*;

pub struct VisualsPlugin;

#[derive(Component)]
pub struct FpsText;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
        ));
        app.init_resource::<CrossHairConfig>();
        app.add_systems(
            Update,
            setup_crosshair.run_if(
                resource_changed::<CrossHairConfig>().or_else(resource_added::<CrossHairConfig>()),
            ),
        );
        app.add_systems(Startup, (config_gizmos, setup))
            .add_systems(PostUpdate, (highlight_target_block, text_update_system));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font_size: 60.0,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: 60.0,
                color: Color::GOLD,
                // If no font is specified, the default font (a minimal subset of FiraMono) will be used.
                ..default()
            }),
        ]),
        FpsText,
    ));
}

fn text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}
