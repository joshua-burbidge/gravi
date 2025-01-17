mod app;
mod handler;
mod helpers;
mod ui;

fn main() {
    let app = {
        #[cfg(feature = "simple")]
        app::orbital::Orbital::new()
        // app::simple::ConstAcceleration::new()
    };

    #[cfg(not(target_arch = "wasm32"))]
    helpers::start(app, 1200, 700, "femtovg app");
    #[cfg(target_arch = "wasm32")]
    helpers::start(app);
}
