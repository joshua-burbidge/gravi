use femtovg::{renderer::WGPURenderer, Canvas, Color, Paint, Path};

pub fn draw_app(canvas: &mut Canvas<WGPURenderer>) {
    let mut path = Path::new();
    path.move_to(0., 0.);
    path.line_to(300., 300.);
    canvas.stroke_path(&path, &Paint::color(Color::white()));
}
