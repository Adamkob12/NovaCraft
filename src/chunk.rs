mod chunk_queue;
mod introduce;
mod misc;
mod spawn;

pub(super) use self::chunk_queue::ComputeChunk;
pub(super) use crate::prelude::*;
use crate::terrain::TerrainConfig;
pub(super) use crate::{blocks::Block, utils::get_neighboring_chunk_cords};
pub(super) use bevy::utils::hashbrown::HashMap;
pub(super) use std::sync::{Arc, RwLock};

use chunk_queue::*;
use introduce::*;
use misc::*;
use rand::prelude::*;
use spawn::*;

// Number of blocks along the y axis
pub const HEIGHT: usize = 56;
// Number of blocks along the z axis
pub const LENGTH: usize = 16;
// Number of blocks along the x axis
pub const WIDTH: usize = 16;
pub const CHUNK_DIMS: (usize, usize, usize) = (WIDTH, HEIGHT, LENGTH);
pub const CHUNK_TOTAL_BLOCKS: usize = HEIGHT * LENGTH * WIDTH;
pub const RENDER_DISTANCE: i32 = 20;

pub const DEFAULT_PBS: Option<PbsParameters> = Some(PbsParameters {
    pbs_value: 0.5,
    min: 0.28,
    smoothing: PbsSmoothing::Custom(2.0),
});

pub type ChunkArr = [Block; CHUNK_TOTAL_BLOCKS];
pub const EMPTY_CHUNK: ChunkArr = [Block::AIR; CHUNK_TOTAL_BLOCKS];

pub type ChunkCords = [i32; 2];
pub type XSpriteMetaData = Box<[(usize, usize, u32, u32); CHUNK_TOTAL_BLOCKS]>;

#[derive(Component)]
pub struct Cords(pub ChunkCords);

#[derive(Component)]
pub struct ToConnect;

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
pub struct ChunkCloseToPlayer;

#[derive(Component)]
pub struct ToIntroduce(pub Vec<(ChunkCords, Direction)>);

#[derive(Component)]
pub struct MainCulledMesh(RwLock<MeshMD<Block>>);

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
    pub pbs: Option<PbsParameters>,
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    #[allow(unused_parens)]
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentChunk([0, 0]))
            .insert_resource(RenderSettings {
                render_distance: RENDER_DISTANCE,
                pbs: DEFAULT_PBS,
            })
            .insert_resource(ChunkMap {
                pos_to_ent: bevy::utils::hashbrown::HashMap::new(),
            })
            .init_resource::<ChunkQueue>();
        app.add_systems(
            Update,
            (reload_all).run_if(
                resource_changed::<RenderSettings>().or_else(resource_changed::<TerrainConfig>()),
            ),
        );
        app.add_systems(
            Update,
            (
                dequeue_all_chunks.run_if(resource_changed::<ChunkQueue>()),
                handle_chunk_spawn_tasks,
                (queue_spawn_despawn_chunks, update_chunks_close_to_player).run_if(
                    resource_changed::<CurrentChunk>()
                        .or_else(resource_added::<CurrentChunk>())
                        .or_else(resource_changed::<RenderSettings>()),
                ),
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

fn update_chunks_close_to_player(
    mut commands: Commands,
    chunk_map: Res<ChunkMap>,
    current_chunk: Res<CurrentChunk>,
    close_chunks_query: Query<Entity, With<ChunkCloseToPlayer>>,
) {
    for ent in close_chunks_query.iter() {
        commands.entity(ent).remove::<ChunkCloseToPlayer>();
    }

    let current_chunk = current_chunk.0;
    for i in -1..=1 {
        for j in -1..=1 {
            if let Some(&ent) = chunk_map
                .pos_to_ent
                .get(&[current_chunk[0] + i, current_chunk[1] + j])
            {
                if ent != Entity::PLACEHOLDER {
                    commands.entity(ent).insert(ChunkCloseToPlayer);
                }
            }
        }
    }
}

fn connect_chunks(
    chunk_map: Res<ChunkMap>,
    chunk_grid_query: Query<&Grid>,
    mut chunk_data_query: Query<(Entity, &mut AdjChunkGrids, &Cords), With<ToConnect>>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    for (entity, mut adj_chunk_grids, cords) in chunk_data_query.iter_mut() {
        let p: f32 = rng.gen();
        if p > 0.2 {
            continue;
        }
        for direction in 1..9 {
            let direction = Direction::from(direction);
            let adj_chunk_cords = get_neighboring_chunk_cords(cords.0, direction);
            if let Some(adj_entity) = chunk_map.pos_to_ent.get(&adj_chunk_cords) {
                if *adj_entity == Entity::PLACEHOLDER {
                    continue;
                }
                if let Ok(Grid(adj_grid)) = chunk_grid_query.get(*adj_entity) {
                    match direction {
                        North => adj_chunk_grids.north = Some(Arc::clone(adj_grid)),
                        South => adj_chunk_grids.south = Some(Arc::clone(adj_grid)),
                        East => adj_chunk_grids.east = Some(Arc::clone(adj_grid)),
                        West => adj_chunk_grids.west = Some(Arc::clone(adj_grid)),
                        NoEast => adj_chunk_grids.no_east = Some(Arc::clone(adj_grid)),
                        NoWest => adj_chunk_grids.no_west = Some(Arc::clone(adj_grid)),
                        SoEast => adj_chunk_grids.so_east = Some(Arc::clone(adj_grid)),
                        SoWest => adj_chunk_grids.so_west = Some(Arc::clone(adj_grid)),
                    }
                }
                if adj_chunk_grids.north.is_some()
                    && adj_chunk_grids.south.is_some()
                    && adj_chunk_grids.west.is_some()
                    && adj_chunk_grids.east.is_some()
                    && adj_chunk_grids.no_east.is_some()
                    && adj_chunk_grids.no_west.is_some()
                    && adj_chunk_grids.so_east.is_some()
                    && adj_chunk_grids.so_west.is_some()
                {
                    commands.entity(entity).remove::<ToConnect>();
                }
            }
        }
    }
}
