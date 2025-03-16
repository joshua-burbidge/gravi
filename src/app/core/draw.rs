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
    // let large_scale_part = canvas_scale * 0.6;
    // let small_scale_part = 1. / canvas_scale;
    // let equalized_scale = large_scale_part + small_scale_part;

    // equalized_scale * width_factor
    width_factor / canvas_scale
}

fn pos_to_canvas(position: &Position, distance_per_px: f32) -> Position {
    Position::new(position.x / distance_per_px, -position.y / distance_per_px)
}
fn convert_length(length: f32, distance_per_px: f32) -> f32 {
    length / distance_per_px
}

fn draw_circle_paint<T: Renderer>(
    canvas: &mut Canvas<T>,
    position: &Position,
    r: f32,
    distance_per_px: f32,
    paint: Paint,
) {
    let mut path = Path::new();
    let px = pos_to_canvas(position, distance_per_px);

    path.circle(px.x, px.y, r);

    canvas.fill_path(&path, &paint);
}

fn draw_circle_green<T: Renderer>(
    canvas: &mut Canvas<T>,
    position: &Position,
    r: f32,
    distance_per_px: f32,
) {
    let paint = Paint::color(Color::rgbf(0., 1., 0.));

    draw_circle_paint(canvas, position, r, distance_per_px, paint);
}

// Draws a circle based on its radius.
// At large scale, the size will be mostly determined by the radius,
// at small scale, the size will be mostly determined by the width_factor so it stays visible.
pub fn draw_circle_by_radius<T: Renderer>(
    canvas: &mut Canvas<T>,
    position: &Position,
    r: f32,
    distance_per_px: f32,
) {
    let fixed_r = convert_length(r, distance_per_px);
    let scaled = scaled_width(canvas, 1.);

    let total_r = if distance_per_px > 2000. {
        fixed_r + scaled
    } else {
        fixed_r
    };
    let total_r = fixed_r; // + scaled;

    draw_circle_green(canvas, position, total_r, distance_per_px);
}

// draws a circle with a radius that scales up and down as you zoom in and out
pub fn draw_circle_scaled<T: Renderer>(
    canvas: &mut Canvas<T>,
    position: &Position,
    width_factor: f32,
    distance_per_px: f32,
) {
    // let width_factor_2 = distance_per_px / 10000. * width_factor;
    let r = scaled_width(canvas, width_factor);

    draw_circle_green(canvas, position, r, distance_per_px);
}

pub fn draw_barycenter<T: Renderer>(
    canvas: &mut Canvas<T>,
    position: &Position,
    width_factor: f32,
    distance_per_px: f32,
) {
    let paint = Paint::color(Color::rgbf(0., 0., 1.));
    let r = scaled_width(canvas, width_factor);

    draw_circle_paint(canvas, position, r, distance_per_px, paint);
}

pub fn draw_line_thru_points<T: Renderer>(
    canvas: &mut Canvas<T>,
    points: Vec<Position>,
    graph_frequency: usize, // number of array elements per graphed point
    distance_per_px: f32,
) {
    let mut points_to_draw = points
        .iter()
        .enumerate()
        .filter(|(i, _)| i % graph_frequency == 0)
        // .rev()
        // .take(10000)
        .map(|(_, pos)| pos);

    let mut line_path = Path::new();
    let initial_pos = points_to_draw.next();
    match initial_pos {
        Some(p) => {
            let canvas_pos = pos_to_canvas(&p, distance_per_px);
            line_path.move_to(canvas_pos.x, canvas_pos.y);
        }
        None => {}
    }
    for pos in points_to_draw {
        let canvas_pos = pos_to_canvas(&pos, distance_per_px);
        line_path.line_to(canvas_pos.x, canvas_pos.y);
    }

    let paint = Paint::color(Color::rgbf(0., 1., 0.)).with_line_width(scaled_width(canvas, 2.));

    canvas.stroke_path(&line_path, &paint);
}

pub fn draw_text<T: Renderer>(
    canvas: &mut Canvas<T>,
    text: String,
    pos: &Position,
    distance_per_px: f32,
) {
    let text_paint = Paint::color(Color::white()).with_font_size(8.0);

    let px = pos_to_canvas(pos, distance_per_px);

    canvas
        .fill_text(px.x, px.y, text, &text_paint)
        .expect("failed to write text");
}
