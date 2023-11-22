use super::{chunkmd::CMMD, *};
use crate::blocks::blockreg::BlockRegistry;

pub(super) fn apply_smooth_lighting_after_update(
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_query: Query<(&Handle<Mesh>, &CMMD, &Parent), With<CubeChunk>>,
    chunks_to_apply_q: Query<(&Children, &AdjChunkGrids, &Grid)>,
    mut removed_components2: RemovedComponents<ToUpdate>,
    breg: Res<BlockRegistry>,
) {
    let breg = Arc::new(breg.into_inner().to_owned());
    for chunk_entity in removed_components2.read() {
        let Ok((mesh_handle, metadata, parent)) = mesh_query.get(chunk_entity) else {
            continue;
        };
        if let Ok((_, acj, Grid(grid))) = chunks_to_apply_q.get(parent.get()) {
            let mesh_ref_mut = meshes.get_mut(mesh_handle).unwrap();
            let north = acj.north.as_ref().unwrap();
            let south = acj.south.as_ref().unwrap();
            let east = acj.east.as_ref().unwrap();
            let west = acj.west.as_ref().unwrap();
            let no_east = acj.no_east.as_ref().unwrap();
            let no_west = acj.no_west.as_ref().unwrap();
            let so_east = acj.so_east.as_ref().unwrap();
            let so_west = acj.so_west.as_ref().unwrap();
            apply_smooth_lighting_with_connected_chunks(
                Arc::clone(&breg).as_ref(),
                mesh_ref_mut,
                &metadata.0.read().unwrap().extract_meshmd().unwrap(),
                CHUNK_DIMS,
                0,
                CHUNK_TOTAL_BLOCKS,
                &*grid.read().unwrap(),
                Some(&*north.read().unwrap()),
                Some(&*south.read().unwrap()),
                Some(&*east.read().unwrap()),
                Some(&*west.read().unwrap()),
                Some(&*no_east.read().unwrap()),
                Some(&*no_west.read().unwrap()),
                Some(&*so_east.read().unwrap()),
                Some(&*so_west.read().unwrap()),
            );
        }
    }
}

pub(super) fn apply_smooth_lighting_after_introduce(
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_query: Query<(&Handle<Mesh>, &CMMD, &Parent), With<CubeChunk>>,
    chunks_to_apply_q: Query<(&Children, &AdjChunkGrids, &Grid)>,
    mut removed_components: RemovedComponents<ToIntroduce>,
    breg: Res<BlockRegistry>,
) {
    let breg = Arc::new(breg.into_inner().to_owned());
    for chunk_entity in removed_components.read() {
        let Ok((children, acj, Grid(grid))) = chunks_to_apply_q.get(chunk_entity) else {
            continue;
        };
        for child in children {
            if let Ok((mesh_handle, metadata, _)) = mesh_query.get(*child) {
                let mesh_ref_mut = meshes.get_mut(mesh_handle).unwrap();
                let north = acj.north.as_ref().unwrap();
                let south = acj.south.as_ref().unwrap();
                let east = acj.east.as_ref().unwrap();
                let west = acj.west.as_ref().unwrap();
                let no_east = acj.no_east.as_ref().unwrap();
                let no_west = acj.no_west.as_ref().unwrap();
                let so_east = acj.so_east.as_ref().unwrap();
                let so_west = acj.so_west.as_ref().unwrap();
                apply_smooth_lighting_with_connected_chunks(
                    Arc::clone(&breg).as_ref(),
                    mesh_ref_mut,
                    &metadata.0.read().unwrap().extract_meshmd().unwrap(),
                    CHUNK_DIMS,
                    0,
                    CHUNK_TOTAL_BLOCKS,
                    &*grid.read().unwrap(),
                    Some(&*north.read().unwrap()),
                    Some(&*south.read().unwrap()),
                    Some(&*east.read().unwrap()),
                    Some(&*west.read().unwrap()),
                    Some(&*no_east.read().unwrap()),
                    Some(&*no_west.read().unwrap()),
                    Some(&*so_east.read().unwrap()),
                    Some(&*so_west.read().unwrap()),
                );
            }
        }
    }
}

pub(super) fn apply_smooth_lighting_edgecases(
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mesh_query: Query<(&Handle<Mesh>, &CMMD, &Parent), With<CubeChunk>>,
    chunks_to_apply_q: Query<
        (Entity, &Children, &AdjChunkGrids, &Grid, &ToApplySL),
        Without<ToConnect>,
    >,
    breg: Res<BlockRegistry>,
) {
    let breg = Arc::new(breg.into_inner().to_owned());
    for (entity, children, acj, Grid(grid), apply_sl) in chunks_to_apply_q.iter() {
        for child in children {
            if let Ok((mesh_handle, metadata, _)) = mesh_query.get(*child) {
                let mesh_ref_mut = meshes.get_mut(mesh_handle).unwrap();
                let north = acj.north.as_ref().unwrap();
                let south = acj.south.as_ref().unwrap();
                let east = acj.east.as_ref().unwrap();
                let west = acj.west.as_ref().unwrap();
                let no_east = acj.no_east.as_ref().unwrap();
                let no_west = acj.no_west.as_ref().unwrap();
                let so_east = acj.so_east.as_ref().unwrap();
                let so_west = acj.so_west.as_ref().unwrap();
                apply_smooth_lighting_with_connected_chunks(
                    Arc::clone(&breg).as_ref(),
                    mesh_ref_mut,
                    &metadata.0.read().unwrap().extract_meshmd().unwrap(),
                    CHUNK_DIMS,
                    apply_sl.0,
                    apply_sl.1,
                    &*grid.read().unwrap(),
                    Some(&*north.read().unwrap()),
                    Some(&*south.read().unwrap()),
                    Some(&*east.read().unwrap()),
                    Some(&*west.read().unwrap()),
                    Some(&*no_east.read().unwrap()),
                    Some(&*no_west.read().unwrap()),
                    Some(&*so_east.read().unwrap()),
                    Some(&*so_west.read().unwrap()),
                );
                commands.entity(entity).remove::<ToApplySL>();
            }
        }
    }
}
