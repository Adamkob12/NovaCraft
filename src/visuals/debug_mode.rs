use bevy::pbr::wireframe::WireframeConfig;

use crate::action::PhysicalPlayer;
use crate::blocks::Block;
use crate::chunk::{Chunk, ChunkChild, Grid, LENGTH, WIDTH};

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub(super) enum DebugMode {
    On,
    #[default]
    Off,
}

pub const TEXT_BACKGROUND_COLOR: Color = Color::Rgba {
    red: 0.81,
    green: 0.23,
    blue: 0.92,
    alpha: 0.5,
};

impl DebugMode {
    fn opposite(&self) -> Self {
        match self {
            DebugMode::On => DebugMode::Off,
            DebugMode::Off => DebugMode::On,
        }
    }
}

#[derive(Resource)]
pub(super) struct DebugModeSettings {
    size: f32,
    render_wireframe: bool,
    render_aabb: bool,
}

#[derive(Component)]
pub(super) struct DebugModeText(DebugText);

#[derive(Component)]
pub(super) struct DebugUI;

pub(super) enum DebugText {
    PlayerPosition,
    TargetBlockType,
    TargetBlockPosition,
}

impl Default for DebugModeSettings {
    fn default() -> Self {
        DebugModeSettings {
            size: 35.0,
            render_aabb: false,
            render_wireframe: false,
        }
    }
}

pub(super) fn setup_debug_mode(
    mut commands: Commands,
    dm_settings: Res<DebugModeSettings>,
    asset_server: Res<AssetServer>,
) {
    let font_handle = asset_server.load("fonts/main_font.ttf");

    // debug player position
    let player_pos = commands
        .spawn((
            DebugModeText(DebugText::PlayerPosition),
            TextBundle::from_sections([
                TextSection::new(
                    "Player Position: ",
                    TextStyle {
                        font_size: dm_settings.size,
                        font: font_handle.clone(),
                        ..Default::default()
                    },
                ),
                TextSection::from_style(TextStyle {
                    font_size: dm_settings.size,
                    font: font_handle.clone(),
                    color: Color::GOLD,
                    ..Default::default()
                }),
            ])
            .with_background_color(TEXT_BACKGROUND_COLOR),
        ))
        .id();
    // debug target block position
    let target_block_pos = commands
        .spawn((
            DebugModeText(DebugText::TargetBlockPosition),
            TextBundle::from_sections([
                TextSection::new(
                    "Target Block Position: ",
                    TextStyle {
                        font_size: dm_settings.size,
                        font: font_handle.clone(),
                        ..Default::default()
                    },
                ),
                TextSection::from_style(TextStyle {
                    font_size: dm_settings.size,
                    font: font_handle.clone(),
                    color: Color::GOLD,
                    ..Default::default()
                }),
            ])
            .with_background_color(TEXT_BACKGROUND_COLOR),
        ))
        .id();
    // debug target block type
    let target_block_type = commands
        .spawn((
            DebugModeText(DebugText::TargetBlockType),
            TextBundle::from_sections([
                TextSection::new(
                    "Target Block Type: ",
                    TextStyle {
                        font_size: dm_settings.size,
                        font: font_handle.clone(),
                        ..Default::default()
                    },
                ),
                TextSection::from_style(TextStyle {
                    font_size: dm_settings.size,
                    font: font_handle,
                    color: Color::GOLD,
                    ..Default::default()
                }),
            ])
            .with_background_color(TEXT_BACKGROUND_COLOR),
        ))
        .id();
    // Have these ^ entities children of parent `DebugUI`
    commands
        .spawn((
            DebugUI,
            NodeBundle {
                style: Style {
                    display: Display::Grid,
                    top: Val::Percent(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .push_children(&[player_pos, target_block_pos, target_block_type]);
}

pub(super) fn update_debug_ui(
    mut debug_ui_text: Query<(&DebugModeText, &mut Text)>,
    player_query: Query<&Transform, With<PhysicalPlayer>>,
    target_block: Res<TargetBlock>,
    grids_query: Query<&Grid, With<Chunk>>,
    parents_query: Query<&Parent, With<ChunkChild>>,
) {
    for (DebugModeText(debug_text_type), mut text) in debug_ui_text.iter_mut() {
        match debug_text_type {
            DebugText::PlayerPosition => {
                if let Ok(player_transform) = player_query.get_single() {
                    text.sections[1].value = format!("{}", player_transform.translation.as_ivec3());
                }
            }
            DebugText::TargetBlockType => {
                if target_block.ignore_flag {
                    text.sections[1].value = format!("{:?}", Block::AIR);
                } else if let Ok(parent) = parents_query.get(target_block.target_entity) {
                    if let Ok(Grid(grid)) = grids_query.get(parent.get()) {
                        text.sections[1].value = format!(
                            "{:?}",
                            grid.read()
                                .unwrap()
                                .get_block_or(target_block.block_pos, Block::AIR)
                        );
                    }
                }
            }
            DebugText::TargetBlockPosition => {
                if target_block.ignore_flag {
                    text.sections[1].value = "NaN".into();
                } else {
                    let tmp = target_block.block_pos.as_vec3();
                    // offset the chunk dims
                    let block_pos = tmp
                        + Vec3::new(
                            target_block.chunk_cords[0] as f32 * WIDTH as f32,
                            0.0,
                            target_block.chunk_cords[1] as f32 * LENGTH as f32,
                        );

                    text.sections[1].value = format!("{}", block_pos.as_ivec3());
                }
            }
        }
    }
}

pub(super) fn hide_debug_mode(
    mut debug_ui: Query<&mut Visibility, With<DebugUI>>,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut aabb_gizmo_config: ResMut<GizmoConfig>,
) {
    if let Ok(mut visibillity) = debug_ui.get_single_mut() {
        *visibillity = Visibility::Hidden;
    }
    wireframe_config.global = false;
    aabb_gizmo_config.aabb.draw_all = false;
}

pub(super) fn show_debug_mode(
    mut debug_ui: Query<&mut Visibility, With<DebugUI>>,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut aabb_gizmo_config: ResMut<GizmoConfig>,
    dm_settings: Res<DebugModeSettings>,
) {
    if let Ok(mut visibillity) = debug_ui.get_single_mut() {
        *visibillity = Visibility::Visible;
    }
    if dm_settings.render_wireframe {
        wireframe_config.global = true;
    }
    if dm_settings.render_aabb {
        aabb_gizmo_config.aabb.draw_all = true;
    }
}

pub(super) fn toggle_debug_mode(
    mut next_state_debug: ResMut<NextState<DebugMode>>,
    current_state_debug: Res<State<DebugMode>>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::T) {
        next_state_debug.set(current_state_debug.get().opposite());
    }
}
