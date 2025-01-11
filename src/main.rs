mod app;
mod egui_renderer;
mod handler;
mod helpers;

fn main() {
    let app = {
        #[cfg(feature = "simple")]
        app::simple::ConstAcceleration::new()
    };

    #[cfg(not(target_arch = "wasm32"))]
    helpers::start(app, 1200, 700, "femtovg app");
    #[cfg(target_arch = "wasm32")]
    helpers::start(app);
}
