use bevy_xpbd_3d::prelude::ComputedCollider;
use parry3d::shape::TriMeshFlags;

use super::{chunkmd::*, *};
use crate::{
    action::meshreg::MeshRegistry, mesh_utils::xsprite_mesh::update_xsprite_mesh, prelude::*,
};

pub(super) fn update_cube_chunks(
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mreg: Res<MeshRegistry>,
    mut chunks_to_update: Query<
        (Entity, &SubChunkMD, &Handle<Mesh>),
        (With<ToUpdate>, With<CubeSubChunk>),
    >,
) {
    let breg = Arc::new(mreg.into_inner().clone());
    for (entity, metadata, mesh_handle) in &mut chunks_to_update {
        let mesh = meshes.get_mut(mesh_handle).unwrap();
        let MetaData::CubeMD(metadata) = &mut *metadata.0.write().unwrap() else {
            continue;
        };
        update_mesh(mesh, metadata, Arc::clone(&breg).as_ref());
        if let Some(aabb) = mesh.compute_aabb() {
            if let Some(mut comm) = commands.get_entity(entity) {
                comm.insert(aabb)
                    .insert(AsyncCollider(ComputedCollider::TriMeshWithFlags(
                        TriMeshFlags::MERGE_DUPLICATE_VERTICES,
                    )))
                    .remove::<ToUpdate>();
            }
        } else {
            warn!("Couldn't compute Aabb for mesh after updating");
        }
    }
}

pub(super) fn update_xsprite_chunks(
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mreg: Res<MeshRegistry>,
    chunks_to_update: Query<
        (Entity, &SubChunkMD, &Handle<Mesh>),
        (With<ToUpdate>, With<XSpriteSubChunk>),
    >,
) {
    let breg = Arc::new(mreg.into_inner().clone());
    for (entity, metadata, mesh_handle) in chunks_to_update.iter() {
        let mesh = meshes.get_mut(mesh_handle).unwrap();
        let MetaData::XSpriteMD(metadata) = &mut *metadata.0.write().unwrap() else {
            continue;
        };
        update_xsprite_mesh(Arc::clone(&breg).as_ref(), mesh, metadata, CHUNK_DIMS);
        if let Some(aabb) = mesh.compute_aabb() {
            if let Some(mut comm) = commands.get_entity(entity) {
                comm.insert(aabb)
                    .insert(AsyncCollider(ComputedCollider::TriMeshWithFlags(
                        TriMeshFlags::MERGE_DUPLICATE_VERTICES,
                    )))
                    .remove::<ToUpdate>();
            }
        } else if mesh.count_vertices() > 0 {
            warn!("Couldn't compute Aabb for mesh after updating");
        } else {
            commands.entity(entity).remove::<ToUpdate>();
        }
    }
}

// pub(super) fn update_chunks(
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut commands: Commands,
//     breg: Res<BlockRegistry>,
//     mut chunks_to_update: Query<(Entity, &mut MainCulledMesh, &Handle<Mesh>), With<ToUpdate>>,
// ) {
//     let breg = Arc::new(breg.into_inner().clone());
//     for (entity, metadata, mesh_handle) in &mut chunks_to_update {
//         let mesh = meshes.get_mut(mesh_handle).unwrap();
//         let metadata = metadata.into_inner().0.get_mut().unwrap();
//         update_mesh(mesh, metadata, Arc::clone(&breg).as_ref());
//         if let Some(aabb) = mesh.compute_aabb() {
//             if let Some(mut comm) = commands.get_entity(entity) {
//                 comm.insert(aabb)
//                     .insert(AsyncCollider(ComputedCollider::TriMeshWithFlags(
//                         TriMeshFlags::MERGE_DUPLICATE_VERTICES,
//                     )))
//                     // .insert(ToApplySL(0, CHUNK_TOTAL_BLOCKS))
//                     .remove::<ToUpdate>();
//             }
//         } else {
//             warn!("Couldn't compute Aabb for mesh after updating");
//         }
//     }
// }
