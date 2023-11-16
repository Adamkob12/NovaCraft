use bevy_xpbd_3d::prelude::ComputedCollider;
use parry3d::shape::TriMeshFlags;

use super::*;
use crate::{action::blockreg::BlockRegistry, prelude::*};

pub(super) fn update_chunks(
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    breg: Res<BlockRegistry>,
    mut chunks_to_update: Query<(Entity, &mut MainCulledMesh, &Handle<Mesh>), With<ToUpdate>>,
) {
    let breg = Arc::new(breg.into_inner().clone());
    for (entity, metadata, mesh_handle) in &mut chunks_to_update {
        let mesh = meshes.get_mut(mesh_handle).unwrap();
        let metadata = metadata.into_inner().0.get_mut().unwrap();
        update_mesh(mesh, metadata, Arc::clone(&breg).as_ref());
        if let Some(aabb) = mesh.compute_aabb() {
            if let Some(mut comm) = commands.get_entity(entity) {
                comm.insert(aabb)
                    .insert(AsyncCollider(ComputedCollider::TriMeshWithFlags(
                        TriMeshFlags::MERGE_DUPLICATE_VERTICES,
                    )))
                    // .insert(ToApplySL(0, CHUNK_TOTAL_BLOCKS))
                    .remove::<ToUpdate>();
            }
        } else {
            warn!("Couldn't compute Aabb for mesh after updating");
        }
    }
}
