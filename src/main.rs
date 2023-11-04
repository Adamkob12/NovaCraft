#![allow(dead_code, unused_imports)]
pub mod prelude {
    pub use bevy::prelude::*;
    pub use bevy::render::mesh::*;
    pub use bevy_meshem::prelude::*;
}

pub mod blocks;
pub mod chunk;
pub mod env;
pub mod inventory;
pub mod meshify_custom_meshes;
pub mod player;
pub mod terrain;
pub mod utils;

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
                    ..Default::default()
                }),
                ..Default::default()
            }),
        chunk::ChunkPlugin,
        player::PlayerPlugin,
        env::EnviornmentPlugin,
    ))
    .init_resource::<BlockRegistry>();

    app.run();
}
