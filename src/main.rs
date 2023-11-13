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
pub mod console;
pub mod env;
pub mod helper_ecs_utils;
pub mod inventory;
pub mod mesh_utils;
pub mod player;
pub mod terrain;
pub mod utils;

use bevy::window::WindowResolution;
use bevy_xpbd_3d::prelude::*;
use blocks::blockreg::BlockRegistry;
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
                    resolution: WindowResolution::new(1820.0, 920.0),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        PhysicsPlugins::default(),
        player::CharacterControllerPlugin,
        chunk::ChunkPlugin,
        player::PlayerPlugin,
        env::EnviornmentPlugin,
        terrain::TerrainPlugin,
        helper_ecs_utils::HelperEcsUtilsPlugin,
        console::GlobalConsolePlugin,
    ))
    .init_resource::<BlockRegistry>();

    app.run();
}
