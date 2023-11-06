use crate::prelude::*;

pub fn generate_xsprite_mesh(
    voxel_dims: [f32; 3],
    texture_atlas_dims: [u32; 2],
    texture: [u32; 2],
    voxel_center: [f32; 3],
    padding: f32,
    default_color_intensity: Option<f32>,
    alpha: f32,
    scale: f32,
) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    debug_assert!(
        0.0 <= scale && scale <= 1.0,
        "scale parameter in generate_xsprite_mesh needs to be in 0.0 - 1.0 range"
    );

    let scale = (scale.min(1.0)).max(0.0);

    let z = voxel_center[2] + voxel_dims[2] / 2.0 * scale;
    let nz = voxel_center[2] - voxel_dims[2] / 2.0 * scale;
    let x = voxel_center[0] + voxel_dims[0] / 2.0 * scale;
    let nx = voxel_center[0] - voxel_dims[0] / 2.0 * scale;
    let y = voxel_center[1] + voxel_dims[1] / 2.0;
    let ny = voxel_center[1] - voxel_dims[1] / 2.0;

    let u: f32 = 1.0 / (texture_atlas_dims[0] as f32);
    let v: f32 = 1.0 / (texture_atlas_dims[1] as f32);

    let padding_u = padding / (texture_atlas_dims[0] as f32);
    let padding_v = padding / (texture_atlas_dims[1] as f32);

    let uvs_tl: [f32; 2] = [
        texture[0] as f32 * u + padding_u,
        texture[1] as f32 * v + padding_v,
    ];
    let uvs_br: [f32; 2] = [
        (texture[0] + 1) as f32 * u - padding_u,
        (texture[1] + 1) as f32 * v - padding_v,
    ];

    // Two quads, in the shape of an X, like in Minecraft. This is used for flowers or grass eg.
    #[rustfmt::skip]
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, 
        vec![
            // First sprite
            [nx, y, z],
            [x, y, nz],
            [x, ny, nz],
            [nx, ny, z],
            // Second sprite
            [nx, y, nz],
            [x, y, z],
            [x, ny, z],
            [nx, ny, nz],
        ]
    );

    #[rustfmt::skip]
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, 
        vec![
            uvs_tl, [uvs_br[0], uvs_tl[1]] ,uvs_br, [uvs_tl[0], uvs_br[1]], 
            uvs_tl, [uvs_br[0], uvs_tl[1]] ,uvs_br, [uvs_tl[0], uvs_br[1]], 
        ]
    );

    if let Some(color) = default_color_intensity {
        #[rustfmt::skip]
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![
                [color, color, color, alpha],
                [color, color, color, alpha],
                [color, color, color, alpha],
                [color, color, color, alpha],
                [color, color, color, alpha],
                [color, color, color, alpha],
                [color, color, color, alpha],
                [color, color, color, alpha],
            ]
        );
    }

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            [0.8, 0.0, 0.8],
            [0.8, 0.0, 0.8],
            [0.8, 0.0, 0.8],
            [0.8, 0.0, 0.8],
            [-0.8, 0.0, 0.8],
            [-0.8, 0.0, 0.8],
            [-0.8, 0.0, 0.8],
            [-0.8, 0.0, 0.8],
        ],
    );

    mesh.set_indices(Some(Indices::U32(vec![0, 1, 3, 2, 3, 1, 4, 5, 7, 6, 7, 5])));

    mesh
}
