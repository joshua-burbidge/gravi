mod egui_ui;
mod handler;
mod helpers;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    helpers::start(1200, 700, "femtovg app");
    #[cfg(target_arch = "wasm32")]
    helpers::start();
}
