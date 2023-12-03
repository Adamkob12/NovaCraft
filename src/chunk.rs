pub mod block_update;
mod chunk_queue;
pub mod chunkmd;
mod falling_block;
mod introduce;
mod misc;
mod smooth_lighting;
mod spawn;
mod update_chunks;

pub use self::chunk_queue::ComputeChunk;
use crate::prelude::*;
use crate::terrain::TerrainConfig;
use crate::{blocks::Block, utils::get_neighboring_chunk_cords};
use bevy::utils::hashbrown::HashMap;
use bevy_xpbd_3d::prelude::AsyncCollider;
use block_update::*;
use smooth_lighting::*;
use std::sync::{Arc, RwLock};

use chunk_queue::*;
pub use falling_block::*;
use introduce::*;
use misc::*;
use rand::prelude::*;
use spawn::*;
use update_chunks::*;

// Number of blocks along the y axis
pub const HEIGHT: u32 = 56;
// Number of blocks along the z axis
pub const LENGTH: u32 = 16;
// Number of blocks along the x axis
pub const WIDTH: u32 = 16;
pub const CHUNK_DIMS: UVec3 = UVec3::new(WIDTH, HEIGHT, LENGTH);
pub const CHUNK_TOTAL_BLOCKS: u32 = HEIGHT * LENGTH * WIDTH;
pub const RENDER_DISTANCE: i32 = 12;

pub const DEFAULT_SL: Option<SmoothLightingParameters> = Some(SmoothLightingParameters {
    intensity: 0.37,
    max: 0.95,
    smoothing: 1.0,
    apply_at_gen: false,
});

pub const CHUNK_TOTAL_BLOCKS_USIZE: usize = CHUNK_TOTAL_BLOCKS as usize;
pub type ChunkArr = ChunkGrid<Block, CHUNK_TOTAL_BLOCKS_USIZE>;
pub const EMPTY_CHUNK: ChunkArr =
    ChunkGrid::new([Block::AIR; CHUNK_TOTAL_BLOCKS as usize], CHUNK_DIMS);

pub type ChunkCords = crate::prelude::ChunkCords;

#[derive(Component)]
pub struct Cords(pub ChunkCords);

#[derive(Component)]
pub struct MainChild(pub Entity);

#[derive(Component)]
pub struct XSpriteChild(pub Entity);

#[derive(Component)]
pub struct ChunkChild;

#[derive(Component)]
pub struct CloseChunk;

#[derive(Component)]
pub struct ToConnect;

#[derive(Component)]
pub struct ToApplySL(pub BlockPos, pub BlockPos);

#[derive(Component)]
pub struct ChunkPhysicalProperties(Vec<crate::player::RigidLayer>);

#[derive(Component)]
pub struct AdjChunkGrids {
    // +z
    pub north: Option<Arc<RwLock<ChunkArr>>>,
    // -z
    pub south: Option<Arc<RwLock<ChunkArr>>>,
    // +x
    pub east: Option<Arc<RwLock<ChunkArr>>>,
    // -x
    pub west: Option<Arc<RwLock<ChunkArr>>>,
    // +z
    pub no_east: Option<Arc<RwLock<ChunkArr>>>,

    pub no_west: Option<Arc<RwLock<ChunkArr>>>,

    pub so_east: Option<Arc<RwLock<ChunkArr>>>,

    pub so_west: Option<Arc<RwLock<ChunkArr>>>,
}

#[derive(Component)]
pub struct Grid(pub Arc<RwLock<ChunkArr>>);

#[derive(Component)]
pub struct Chunk;

#[derive(Component)]
pub struct ToUpdate;

#[derive(Component)]
pub struct ToIntroduce(pub Vec<(ChunkCords, Direction)>);

#[derive(Component)]
pub struct CubeChunk;

#[derive(Component)]
pub struct XSpriteChunk;

#[derive(Resource)]
pub struct BlockMaterial(Handle<StandardMaterial>);

#[derive(Resource)]
pub struct XSpriteMaterial(Handle<StandardMaterial>);

#[derive(Resource, Default)]
pub struct ChunkMap {
    pub pos_to_ent: HashMap<ChunkCords, Entity>,
}

#[derive(Resource, PartialEq)]
pub struct CurrentChunk(pub ChunkCords);

#[derive(Resource)]
pub struct RenderSettings {
    pub render_distance: i32,
    pub sl: Option<SmoothLightingParameters>,
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    #[allow(unused_parens)]
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentChunk([0, 0].into()))
            .insert_resource(RenderSettings {
                render_distance: RENDER_DISTANCE,
                sl: DEFAULT_SL,
            })
            .insert_resource(ChunkMap {
                pos_to_ent: bevy::utils::hashbrown::HashMap::with_capacity(
                    (RENDER_DISTANCE * RENDER_DISTANCE) as usize,
                ),
            })
            .init_resource::<ChunkQueue>()
            .insert_resource(LockChunkUpdate::unlocked());
        app.add_systems(
            PreUpdate,
            (reload_all.run_if(
                resource_changed::<RenderSettings>().or_else(resource_changed::<TerrainConfig>()),
            ),),
        );
        app.add_systems(
            Update,
            (
                queue_spawn_despawn_chunks,
                dequeue_all_chunks.run_if(resource_changed::<ChunkQueue>()),
                handle_chunk_spawn_tasks,
                ((update_cube_chunks, update_xsprite_chunks), apply_deferred,
                (apply_smooth_lighting_after_update, apply_smooth_lighting_edgecases))
                    .chain().run_if(resource_equals(LockChunkUpdate::unlocked())),
            ).run_if(in_state(AssetLoadingState::Loaded)),
        )
        .add_systems(PostUpdate, (update_close_chunks, insert_collider_for_close_chunks))
        .add_systems(
            PostUpdate,
            ((connect_chunks, introduce_neighboring_chunks, apply_smooth_lighting_after_introduce).run_if(
                not(any_with_component::<ComputeChunk>())/* .and_then(resource_changed::<OneIn2>()) */,
            ),),
        )
        .add_systems(PostStartup, setup_texture);
    }
}

fn setup_texture(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture_handle = asset_server.load("blocks.png");
    let blocks_mat = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        reflectance: 0.0,
        alpha_mode: AlphaMode::Mask(0.3),
        perceptual_roughness: 0.85,
        ..default()
    });
    commands.insert_resource(BlockMaterial(blocks_mat));

    let xsprite_mat = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        reflectance: 0.0,
        alpha_mode: AlphaMode::Mask(0.1),
        perceptual_roughness: 0.85,
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    commands.insert_resource(XSpriteMaterial(xsprite_mat));
}

impl AdjChunkGrids {
    pub fn get_grid_at_direction(&self, dir: crate::prelude::Direction) -> &Arc<RwLock<ChunkArr>> {
        let grid_to_return = match dir {
            North if self.north.is_some() => self.north.as_ref().unwrap(),
            South if self.south.is_some() => self.north.as_ref().unwrap(),
            East if self.east.is_some() => self.north.as_ref().unwrap(),
            West if self.west.is_some() => self.north.as_ref().unwrap(),
            NoEast if self.no_east.is_some() => self.north.as_ref().unwrap(),
            NoWest if self.no_west.is_some() => self.north.as_ref().unwrap(),
            SoEast if self.so_east.is_some() => self.north.as_ref().unwrap(),
            SoWest if self.so_west.is_some() => self.north.as_ref().unwrap(),
            _ => panic!("Can't get grid becuase it's not connected."),
        };
        grid_to_return
    }
}
