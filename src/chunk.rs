mod chunk_queue;
mod introduce;
mod misc;
mod smooth_lighting;
mod spawn;
mod update_chunks;

pub(super) use self::chunk_queue::ComputeChunk;
pub(super) use crate::prelude::*;
use crate::terrain::TerrainConfig;
pub(super) use crate::{blocks::Block, utils::get_neighboring_chunk_cords};
pub(super) use bevy::utils::hashbrown::HashMap;
use bevy_xpbd_3d::prelude::AsyncCollider;
use smooth_lighting::*;
pub(super) use std::sync::{Arc, RwLock};

use chunk_queue::*;
use introduce::*;
use misc::*;
use rand::prelude::*;
use spawn::*;
use update_chunks::*;

// Number of blocks along the y axis
pub const HEIGHT: usize = 56;
// Number of blocks along the z axis
pub const LENGTH: usize = 16;
// Number of blocks along the x axis
pub const WIDTH: usize = 16;
pub const CHUNK_DIMS: (usize, usize, usize) = (WIDTH, HEIGHT, LENGTH);
pub const CHUNK_TOTAL_BLOCKS: usize = HEIGHT * LENGTH * WIDTH;
pub const RENDER_DISTANCE: i32 = 16;

pub const DEFAULT_SL: Option<SmoothLightingParameters> = Some(SmoothLightingParameters {
    intensity: 0.35,
    max: 0.8,
    smoothing: 1.2,
    apply_at_gen: false,
});

pub type ChunkArr = [Block; CHUNK_TOTAL_BLOCKS];
pub const EMPTY_CHUNK: ChunkArr = [Block::AIR; CHUNK_TOTAL_BLOCKS];

pub type ChunkCords = [i32; 2];
pub type XSpriteMetaData = Box<[(usize, usize, u32, u32); CHUNK_TOTAL_BLOCKS]>;

#[derive(Component)]
pub struct Cords(pub ChunkCords);

#[derive(Component)]
pub struct MainChild(pub Entity);

#[derive(Component)]
pub struct ToConnect;

#[derive(Component)]
// lower bound and upper bound
pub struct ToApplySL(pub usize, pub usize);

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
pub struct CollidableChunk;

#[derive(Component)]
pub struct ToUpdate;

#[derive(Component)]
pub struct ToIntroduce(pub Vec<(ChunkCords, Direction)>);

#[derive(Component)]
pub struct MainCulledMesh(pub RwLock<MeshMD<Block>>);

#[derive(Component)]
pub struct XSpriteMesh(RwLock<XSpriteMetaData>);

#[derive(Resource)]
pub struct BlockMaterial(Handle<StandardMaterial>);

#[derive(Resource)]
pub struct XSpriteMaterial(Handle<StandardMaterial>);

#[derive(Resource, Default)]
pub struct ChunkMap {
    pub pos_to_ent: HashMap<[i32; 2], Entity>,
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
        app.insert_resource(CurrentChunk([0, 0]))
            .insert_resource(RenderSettings {
                render_distance: RENDER_DISTANCE,
                sl: DEFAULT_SL,
            })
            .insert_resource(ChunkMap {
                pos_to_ent: bevy::utils::hashbrown::HashMap::with_capacity(
                    (RENDER_DISTANCE * RENDER_DISTANCE) as usize,
                ),
            })
            .init_resource::<ChunkQueue>();
        app.add_systems(
            PreUpdate,
            (reload_all.run_if(
                resource_changed::<RenderSettings>().or_else(resource_changed::<TerrainConfig>()),
            ),),
        );
        app.add_systems(
            Update,
            (
                dequeue_all_chunks.run_if(resource_changed::<ChunkQueue>()),
                handle_chunk_spawn_tasks,
                apply_smooth_lighting_system,
                queue_spawn_despawn_chunks,
                update_chunks,
                    // .run_if(resource_changed::<CurrentChunk>()
                    //     .or_else(not(any_with_component::<ChunkCloseToPlayer>())))
                
            ),
        )
        .add_systems(
            PostUpdate,
            ((connect_chunks, introduce_neighboring_chunks).run_if(
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
        let grid_to_return = 
        match dir {
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
