#![allow(dead_code, unused_imports)]
pub mod prelude {
    pub use crate::direction::{Direction, Direction::*};
    pub use crate::helper_ecs_utils::*;
    pub use bevy::prelude::*;
    pub use bevy::render::mesh::*;
    pub use bevy_meshem::prelude::*;
}

pub mod action;
pub mod blocks;
pub mod chunk;
pub mod direction;
pub mod env;
pub mod helper_ecs_utils;
pub mod inventory;
pub mod meshify_custom_meshes;
pub mod player;
pub mod terrain;
pub mod utils;

use bevy::window::WindowResolution;
use blocks::blockreg::BlockRegistry;
use helper_ecs_utils::*;
use prelude::*;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resizable: false,
                    mode: bevy::window::WindowMode::Windowed,
                    resolution: WindowResolution::new(1200.0, 700.0),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        chunk::ChunkPlugin,
        player::PlayerPlugin,
        env::EnviornmentPlugin,
        helper_ecs_utils::HelperEcsUtilsPlugin,
    ))
    .init_resource::<BlockRegistry>();

    app.run();
}
