use bevy::{
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

use crate::world_config::GameWorldConfig;

pub struct SkyboxPlugin;

/// Tracks the pending skybox image handle until it finishes loading.
#[derive(Resource)]
struct PendingSkybox(Handle<Image>);

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start_loading_skybox);
        app.add_systems(Update, attach_skybox_when_ready);
    }
}

fn start_loading_skybox(
    mut commands: Commands,
    world_config: Res<GameWorldConfig>,
    asset_server: Res<AssetServer>,
) {
    let handle = asset_server.load(&world_config.world_assets.skybox_path);
    commands.insert_resource(PendingSkybox(handle));
}

fn attach_skybox_when_ready(
    mut commands: Commands,
    pending: Option<Res<PendingSkybox>>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    camera_query: Query<Entity, (With<Camera3d>, Without<Skybox>)>,
) {
    let Some(pending) = pending else {
        return;
    };
    if !asset_server.is_loaded_with_dependencies(&pending.0) {
        return;
    }

    let skybox_handle = pending.0.clone();
    if let Some(image) = images.get_mut(&skybox_handle) {
        prepare_skybox_cubemap(image);
    }

    if let Ok(camera_entity) = camera_query.single() {
        commands.entity(camera_entity).insert(Skybox {
            image: skybox_handle,
            brightness: 100.0,
            rotation: Quat::IDENTITY,
        });
        info!("Skybox attached to camera");
    }

    commands.remove_resource::<PendingSkybox>();
}

/// Convert a loaded skybox image to a cube texture.
/// Handles equirectangular (2:1), stacked cubemap (1:6), or other stacked layouts.
pub fn prepare_skybox_cubemap(image: &mut Image) {
    let w = image.width();
    let h = image.height();

    if w == 0 || h == 0 {
        warn!("Skybox image has zero dimensions, skipping cubemap conversion");
        return;
    }

    if h == w * 6 {
        image
            .reinterpret_stacked_2d_as_array(6)
            .expect("Failed to reinterpret stacked cubemap");
    } else if w == h * 2 {
        info!("Converting equirectangular skybox ({}x{}) to cubemap", w, h);
        equirect_to_cubemap(image);
    } else {
        warn!(
            "Skybox image has unexpected aspect ratio ({}x{}), attempting stacked reinterpret",
            w, h
        );
        let layers = h / w;
        if layers > 0 && h == w * layers {
            image
                .reinterpret_stacked_2d_as_array(layers)
                .expect("Failed to reinterpret");
        }
    }

    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
}

/// Convert an equirectangular (2:1) image in-place to a stacked cubemap.
///
/// Uses bilinear interpolation for smooth sampling.
/// Face order: +X, -X, +Y, -Y, +Z, -Z (standard cubemap convention).
fn equirect_to_cubemap(image: &mut Image) {
    let src_w = image.width() as usize;
    let src_h = image.height() as usize;
    let bpp = image
        .texture_descriptor
        .format
        .block_copy_size(None)
        .unwrap_or(16) as usize;
    let src_data = image.data.clone().expect("Skybox image has no data");
    let floats_per_pixel = bpp / 4;

    let face_size = (src_h / 2).min(2048);
    let out_h = face_size * 6;

    let mut out_data = vec![0u8; face_size * out_h * bpp];

    for (face_idx, face_dirs) in CUBE_FACE_DIRS.iter().enumerate() {
        let y_offset = face_idx * face_size;
        for row in 0..face_size {
            for col in 0..face_size {
                let u = (col as f32 + 0.5) / face_size as f32 * 2.0 - 1.0;
                let v = (row as f32 + 0.5) / face_size as f32 * 2.0 - 1.0;

                let dir = face_dirs(u, v);
                let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
                let (dx, dy, dz) = (dir[0] / len, dir[1] / len, dir[2] / len);

                let theta = dx.atan2(dz);
                let phi = dy.asin();
                let eq_u = 0.5 + theta / (2.0 * std::f32::consts::PI);
                let eq_v = 0.5 - phi / std::f32::consts::PI;

                let fx = eq_u * src_w as f32 - 0.5;
                let fy = eq_v * src_h as f32 - 0.5;
                let x0 = (fx.floor() as isize).rem_euclid(src_w as isize) as usize;
                let y0 = (fy.floor() as isize).clamp(0, src_h as isize - 1) as usize;
                let x1 = (x0 + 1) % src_w;
                let y1 = (y0 + 1).min(src_h - 1);
                let frac_x = fx - fx.floor();
                let frac_y = fy - fy.floor();

                let dst_offset = ((y_offset + row) * face_size + col) * bpp;

                for c in 0..floats_per_pixel {
                    let s00 = read_f32(&src_data, (y0 * src_w + x0) * bpp + c * 4);
                    let s10 = read_f32(&src_data, (y0 * src_w + x1) * bpp + c * 4);
                    let s01 = read_f32(&src_data, (y1 * src_w + x0) * bpp + c * 4);
                    let s11 = read_f32(&src_data, (y1 * src_w + x1) * bpp + c * 4);

                    let top = s00 + (s10 - s00) * frac_x;
                    let bot = s01 + (s11 - s01) * frac_x;
                    let val = top + (bot - top) * frac_y;
                    let bytes = val.to_le_bytes();
                    let o = dst_offset + c * 4;
                    out_data[o..o + 4].copy_from_slice(&bytes);
                }
            }
        }
    }

    image.data = Some(out_data);
    image.texture_descriptor.size.width = face_size as u32;
    image.texture_descriptor.size.height = out_h as u32;
    image
        .reinterpret_stacked_2d_as_array(6)
        .expect("Failed to reinterpret converted cubemap");
}

fn read_f32(data: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

/// Direction functions for each cube face: +X, -X, +Y, -Y, +Z, -Z.
const CUBE_FACE_DIRS: [fn(f32, f32) -> [f32; 3]; 6] = [
    |u, v| [1.0, -v, -u],  // +X
    |u, v| [-1.0, -v, u],  // -X
    |u, v| [u, 1.0, v],    // +Y
    |u, v| [u, -1.0, -v],  // -Y
    |u, v| [u, -v, 1.0],   // +Z
    |u, v| [-u, -v, -1.0], // -Z
];
