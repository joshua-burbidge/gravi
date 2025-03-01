use femtovg::{Canvas, Color, Paint, Path, Renderer};

use super::physics::Position;

pub fn get_scale<T: Renderer>(canvas: &Canvas<T>) -> f32 {
    let transform_matrix = canvas.transform().0;
    let scale_opt = transform_matrix.get(0);

    match scale_opt {
        Some(scale) => *scale,
        None => 1.,
    }
}

pub fn scaled_width<T: Renderer>(canvas: &Canvas<T>, width_factor: f32) -> f32 {
    let canvas_scale = get_scale(canvas);

    // balance width when scale is small and large
    let equalized_scale = canvas_scale + 1. / canvas_scale;

    equalized_scale * width_factor
}

fn pos_to_canvas(position: &Position, distance_per_px: f32) -> Position {
    Position::new(position.x / distance_per_px, -position.y / distance_per_px)
}
fn convert_length(length: f32, distance_per_px: f32) -> f32 {
    length / distance_per_px
}

fn draw_circle<T: Renderer>(canvas: &mut Canvas<T>, position: &Position, r: f32) {
    let distance_per_px = 10.;

    let mut path = Path::new();
    let px = pos_to_canvas(position, distance_per_px);

    path.circle(px.x, px.y, r);

    let paint = Paint::color(Color::rgbf(0., 1., 0.));
    canvas.fill_path(&path, &paint);
}

// draws a circle with a fixed radius
pub fn draw_circle_fixed<T: Renderer>(canvas: &mut Canvas<T>, position: &Position, r: f32) {
    let distance_per_px = 10.;

    let scaled_r = convert_length(r, distance_per_px);

    draw_circle(canvas, position, scaled_r);
}
// draws a circle with a radius that scales up and down as you zoom in and out
pub fn draw_circle_scaled<T: Renderer>(
    canvas: &mut Canvas<T>,
    position: &Position,
    width_factor: f32,
) {
    let r = scaled_width(canvas, width_factor);

    draw_circle(canvas, position, r);
}
