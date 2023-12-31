#[allow(unused_imports)]
use bevy::pbr::wireframe::Wireframe;

use crate::{blocks::meshreg::MeshRegistry, utils::chunk_distance};

use super::{
    chunk_queue::ChunkQueue,
    chunkmd::{MetaData, SubChunkMD},
    *,
};

// only run when CurrentChunk has changed
pub fn queue_spawn_despawn_chunks(
    current_chunk: Res<CurrentChunk>,
    mut chunk_queue: ResMut<ChunkQueue>,
    render_settings: Res<RenderSettings>,
    mut chunk_map: ResMut<ChunkMap>,
    mut commands: Commands,
) {
    let render_distance = render_settings.render_distance;
    let current_chunk = current_chunk.0;

    let chunks_to_despawn: HashMap<ChunkCords, Entity> = chunk_map
        .pos_to_ent
        .extract_if(|k, _v| chunk_distance(*k, current_chunk) > render_distance)
        .collect();
    let chunks_to_despawn: Vec<Entity> = chunks_to_despawn.into_values().collect();
    chunks_to_despawn
        .iter()
        .filter(|ent| **ent != Entity::PLACEHOLDER)
        .for_each(|ent| commands.entity(*ent).despawn_recursive());

    for u in -render_distance..=render_distance {
        for v in -render_distance..=render_distance {
            let cords = [current_chunk[0] + u, current_chunk[1] + v].into();
            if !chunk_map.pos_to_ent.contains_key(&cords) {
                chunk_queue.enqueue(cords);
            }
        }
    }
}

pub fn dequeue_all_chunks(
    commands: Commands,
    mut chunk_map: ResMut<ChunkMap>,
    mreg: Res<MeshRegistry>,
    mut chunk_queue: ResMut<ChunkQueue>,
    current_chunk: Res<CurrentChunk>,
    render_settings: Res<RenderSettings>,
    terrain_config: Res<TerrainConfig>,
) {
    let breg = Arc::new(mreg.into_inner().to_owned());
    chunk_queue.dequeue_all(
        &mut chunk_map,
        commands,
        &breg,
        Some(|x: &ChunkCords| {
            chunk_distance(*x, current_chunk.0) < render_settings.render_distance
        }),
        &render_settings,
        &terrain_config,
    );
}

pub fn handle_chunk_spawn_tasks(
    mut task_query: Query<(Entity, &mut ComputeChunk)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    blocks_mat: Res<BlockMaterial>,
    xsprite_mat: Res<XSpriteMaterial>,
    current_chunk: Res<CurrentChunk>,
    mut chunk_map: ResMut<ChunkMap>,
    render_settings: Res<RenderSettings>,
) {
    let current_chunk_cords = current_chunk.0;
    for (ent, mut task) in task_query.iter_mut() {
        if let Some(Some(((culled_mesh, metadata), grid, cords, (xsprite_mesh, data)))) =
            futures_lite::future::block_on(futures_lite::future::poll_once(&mut task.0))
        {
            // Remove the task so we don't poll it again
            commands.entity(ent).remove::<ComputeChunk>();
            // If while the task was computing, the player left the area from which the chunk
            // should be in, we just don't spawn the chunk.
            if (current_chunk_cords[0] - cords[0]).abs() > render_settings.render_distance
                || (current_chunk_cords[1] - cords[1]).abs() > render_settings.render_distance
            {
                chunk_map.pos_to_ent.remove(&cords);
                continue;
            }
            if let Some(chunk_entity) = chunk_map.pos_to_ent.get_mut(&cords) {
                let culled_mesh_handle = meshes.add(culled_mesh);
                let xsprite_mesh_handle = meshes.add(xsprite_mesh);
                let transform = Transform::from_xyz(
                    (cords[0] * WIDTH as i32) as f32,
                    0.0,
                    (cords[1] * LENGTH as i32) as f32,
                );
                let culled_mesh_child = commands
                    .spawn((
                        // MainCulledMesh(metadata.into()),
                        SubChunkMD(MetaData::CubeMD(metadata).into()),
                        CubeSubChunk,
                        ChunkRigidLayers(vec![crate::player::RigidLayer::Ground]),
                        Subchunk,
                        PbrBundle {
                            mesh: culled_mesh_handle,
                            material: blocks_mat.0.clone(),
                            ..Default::default()
                        },
                    ))
                    .id();
                let xsprite_mesh_child = commands
                    .spawn((
                        PbrBundle {
                            mesh: xsprite_mesh_handle,
                            material: xsprite_mat.0.clone(),
                            ..Default::default()
                        },
                        ChunkRigidLayers(vec![crate::player::RigidLayer::GroundNonCollidable]),
                        Subchunk,
                        // XSpriteMesh(RwLock::new(data)),
                        SubChunkMD(MetaData::XSpriteMD(data).into()),
                        XSpriteSubChunk,
                    ))
                    .id();
                let entity = commands
                    .spawn((
                        ParentChunk,
                        CubeChild(culled_mesh_child),
                        XSpriteChild(xsprite_mesh_child),
                        Grid(Arc::new(RwLock::new(grid))),
                        AdjChunkGrids {
                            north: None,
                            south: None,
                            west: None,
                            east: None,
                            no_east: None,
                            no_west: None,
                            so_east: None,
                            so_west: None,
                        },
                        Cords(cords),
                        ToIntroduce(vec![
                            (get_neighboring_chunk_cords(cords, East), East),
                            (get_neighboring_chunk_cords(cords, West), West),
                            (get_neighboring_chunk_cords(cords, North), North),
                            (get_neighboring_chunk_cords(cords, South), South),
                            (get_neighboring_chunk_cords(cords, NoEast), NoEast),
                            (get_neighboring_chunk_cords(cords, NoWest), NoWest),
                            (get_neighboring_chunk_cords(cords, SoEast), SoEast),
                            (get_neighboring_chunk_cords(cords, SoWest), SoWest),
                        ]),
                        ToConnect,
                        SpatialBundle {
                            transform,
                            ..Default::default()
                        },
                    ))
                    .id();

                commands
                    .entity(entity)
                    .push_children(&[culled_mesh_child, xsprite_mesh_child]);
                *chunk_entity = entity;
            }
        }
    }
}
