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

pub fn draw_line_thru_points<T: Renderer>(
    canvas: &mut Canvas<T>,
    points: Vec<Position>,
    graph_frequency: usize, // number of array elements per graphed point
) {
    let dist_per_px = 10.;

    let mut points_to_draw = points
        .iter()
        .enumerate()
        .filter(|(i, _)| i % graph_frequency == 0)
        .map(|(_, pos)| pos.clone());

    let mut line_path = Path::new();
    let initial_pos = points_to_draw.next();
    match initial_pos {
        Some(p) => {
            let canvas_pos = pos_to_canvas(&p, dist_per_px);
            line_path.move_to(canvas_pos.x, canvas_pos.y);
        }
        None => {}
    }
    for pos in points_to_draw {
        let canvas_pos = pos_to_canvas(&pos, dist_per_px);
        line_path.line_to(canvas_pos.x, canvas_pos.y);
    }

    let paint = Paint::color(Color::rgbf(0., 1., 0.)).with_line_width(scaled_width(canvas, 1.));

    canvas.stroke_path(&line_path, &paint);
}
