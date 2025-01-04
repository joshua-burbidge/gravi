#[cfg(feature = "wgpu")]
pub mod wgpu;

pub fn start(
    #[cfg(not(target_arch = "wasm32"))] width: u32,
    #[cfg(not(target_arch = "wasm32"))] height: u32,
    #[cfg(not(target_arch = "wasm32"))] title: &'static str,
) {
    #[cfg(feature = "wgpu")]
    use wgpu::start_wgpu as async_start;
    #[cfg(not(target_arch = "wasm32"))]
    spin_on::spin_on(async_start(width, height, title));
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async_start());
}
