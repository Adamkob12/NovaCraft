use bevy::window::PrimaryWindow;

use crate::prelude::*;

#[derive(Resource)]
pub struct CrossHairConfig {
    color: Color,
    style: CrossHairStyle,
    size: f32,
}

#[allow(dead_code)]
pub enum CrossHairStyle {
    Cross,
    Dot,
}

impl Default for CrossHairConfig {
    fn default() -> Self {
        CrossHairConfig {
            size: 30.0,
            color: Color::WHITE,
            style: CrossHairStyle::Cross,
        }
    }
}

#[derive(Component)]
pub struct Crosshair;

pub(super) fn setup_crosshair(
    ch_config: Res<CrossHairConfig>,
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    current_crosshair: Query<Entity, With<Crosshair>>,
) {
    if let Ok(crosshair) = current_crosshair.get_single() {
        commands.entity(crosshair).despawn();
    }
    if let Ok(window) = window.get_single() {
        let (window_width, window_height) = (window.resolution.width(), window.resolution.height());
        commands.spawn(Crosshair);
        match ch_config.style {
            CrossHairStyle::Cross => {
                commands.spawn(
                    TextBundle::from_section(
                        format!("+"),
                        TextStyle {
                            font_size: ch_config.size,
                            color: ch_config.color,
                            ..default()
                        },
                    )
                    .with_style(Style {
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        top: Val::Px(window_height / 2.0 - ch_config.size / 2.0),
                        left: Val::Px(window_width / 2.0 - ch_config.size / 2.0),
                        ..default()
                    }),
                );
            }
            CrossHairStyle::Dot => {
                commands.spawn(
                    TextBundle::from_section(
                        format!("."),
                        TextStyle {
                            font_size: ch_config.size,
                            color: ch_config.color,
                            ..default()
                        },
                    )
                    .with_style(Style {
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        top: Val::Px(window_height / 2.0 - ch_config.size / 2.0),
                        left: Val::Px(window_width / 2.0 - ch_config.size / 2.0),
                        ..default()
                    }),
                );
            }
        }
    }
}
