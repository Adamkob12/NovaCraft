mod block_highlight;

pub(super) use crate::player::TargetBlock;
pub(super) use crate::prelude::*;
use block_highlight::*;

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, config_gizmos)
            .add_systems(PostUpdate, highlight_target_block);
    }
}
