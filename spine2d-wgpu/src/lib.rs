//! wgpu renderer integration for [`spine2d`].

#![forbid(unsafe_code)]

#[cfg(all(feature = "wgpu-29", feature = "wgpu-30"))]
compile_error!("features `wgpu-29` and `wgpu-30` are mutually exclusive");
#[cfg(not(any(feature = "wgpu-29", feature = "wgpu-30")))]
compile_error!("enable exactly one wgpu backend feature: `wgpu-29` or `wgpu-30`");

#[cfg(feature = "wgpu-29")]
pub use wgpu_29 as wgpu;
#[cfg(feature = "wgpu-30")]
pub use wgpu_30 as wgpu;

mod renderer;

pub use renderer::*;

/// Presents a surface texture using the API selected by the active wgpu backend feature.
pub fn present_surface_texture(queue: &wgpu::Queue, surface_texture: wgpu::SurfaceTexture) {
    #[cfg(feature = "wgpu-29")]
    {
        let _ = queue;
        surface_texture.present();
    }
    #[cfg(feature = "wgpu-30")]
    queue.present(surface_texture);
}
