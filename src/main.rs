mod app;
mod handler;
mod helpers;
mod ui;

fn main() {
    env_logger::builder().format_timestamp(None).init();

    let app = {
        #[cfg(feature = "simple")]
        app::orbital::Orbital::new()
        // app::simple::ConstAcceleration::new()
    };

    #[cfg(not(target_arch = "wasm32"))]
    helpers::start(app, 1600, 1000, "femtovg app");
    #[cfg(target_arch = "wasm32")]
    helpers::start(app);
}
