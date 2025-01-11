pub mod simple;
pub use simple::App;

pub fn create_app() -> App {
    App::new()
}
