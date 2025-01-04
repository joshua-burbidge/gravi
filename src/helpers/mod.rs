use std::sync::Arc;

use ::wgpu::{Device, Queue, Surface, SurfaceTexture};

// the purpose of this trait is to have separate implementations
// for wgpu and opengl
pub trait WindowSurface {
    type Renderer: femtovg::Renderer + 'static;
    // resize only used in non-wasm
    fn resize(&mut self, width: u32, height: u32);
    fn present(
        &self,
        canvas: &mut femtovg::Canvas<Self::Renderer>,
        surface_texture: &SurfaceTexture,
    );
    fn get_device(&self) -> &Arc<Device>;
    fn get_queue(&self) -> &Arc<Queue>;
    fn get_surface(&self) -> &Surface<'static>;
}

#[cfg(not(feature = "wgpu"))]
mod opengl;

#[cfg(feature = "wgpu")]
mod wgpu;

pub fn start(
    #[cfg(not(target_arch = "wasm32"))] width: u32,
    #[cfg(not(target_arch = "wasm32"))] height: u32,
    #[cfg(not(target_arch = "wasm32"))] title: &'static str,
) {
    #[cfg(not(feature = "wgpu"))]
    use opengl::start_opengl as async_start;
    #[cfg(feature = "wgpu")]
    use wgpu::start_wgpu as async_start;
    #[cfg(not(target_arch = "wasm32"))]
    spin_on::spin_on(async_start(width, height, title));
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async_start());
}
