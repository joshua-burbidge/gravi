pub mod core;
pub mod simple;
use femtovg::{renderer::WGPURenderer, Canvas};

pub trait App {
    fn run(&mut self);
    fn draw(&mut self, canvas: &mut Canvas<WGPURenderer>);
    fn ui(&mut self, ctx: &egui::Context);
    fn panel_width(&self) -> f32;
}
