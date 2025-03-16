use femtovg::{Canvas, Renderer};
use resource::resource;

use crate::app::App;

pub mod wgpu;

pub fn start<A: App + 'static>(
    app: A,
    #[cfg(not(target_arch = "wasm32"))] width: u32,
    #[cfg(not(target_arch = "wasm32"))] height: u32,
    #[cfg(not(target_arch = "wasm32"))] title: &'static str,
) {
    use wgpu::start_wgpu as async_start;

    #[cfg(not(target_arch = "wasm32"))]
    spin_on::spin_on(async_start(app, width, height, title));
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async_start(app));
}

fn init_canvas<T: Renderer>(canvas: &mut Canvas<T>) {
    canvas
        .add_font_mem(&resource!("assets/roboto-regular.ttf"))
        .expect("Cannot add font");
}
