pub mod core;
pub mod orbital;
pub mod simple;
use femtovg::{renderer::WGPURenderer, Canvas};

pub trait App {
    fn run(&mut self);
    fn draw(&self, canvas: &mut Canvas<WGPURenderer>);
    fn ui(&mut self, ctx: &egui::Context);
    fn panel_width(&self) -> f32;
    fn focused_pos(&self) -> Option<(f32, f32)>;
}
