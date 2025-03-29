use std::ops::Neg;

use egui::emath::round_to_decimals;
use femtovg::{Canvas, Color, Paint, Path, Renderer};

use crate::app::orbital::body::Body;

use super::physics::{Axis, Position};

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
    width_factor / canvas_scale
}

fn pos_to_canvas(position: &Position, distance_per_px: f32) -> Position {
    Position::new(position.x / distance_per_px, -position.y / distance_per_px)
}
fn convert_length(length: f32, distance_per_px: f32) -> f32 {
    length / distance_per_px
}

fn white<T: Renderer>(canvas: &Canvas<T>) -> Paint {
    Paint::color(Color::white()).with_line_width(scaled_width(canvas, 1.))
}

pub fn draw_line_px<T: Renderer>(canvas: &mut Canvas<T>, start_px: &Position, end_px: &Position) {
    let mut path = Path::new();

    path.move_to(start_px.x, start_px.y);
    path.line_to(end_px.x, end_px.y);

    let paint = white(canvas);
    canvas.stroke_path(&path, &paint);
}

// one dimension is distance, the other is a fixed px length
fn draw_tick<T: Renderer>(
    canvas: &mut Canvas<T>,
    axis: &Axis,
    axis_distance: f32,
    distance_per_px: f32,
) {
    let tick_length = scaled_width(canvas, 20.);
    let tick_dist_from_axis = tick_length / 2.;
    let (tick_start, tick_end) = (-tick_dist_from_axis, tick_dist_from_axis);

    let (start_px, end_px) = match axis {
        Axis::X => {
            let tick_pos = Position::new(axis_distance, 0.);
            let tick_px = pos_to_canvas(&tick_pos, distance_per_px).x;

            let start = Position::new(tick_px, tick_start);
            let end = Position::new(tick_px, tick_end);
            (start, end)
        }
        Axis::Y => {
            let tick_pos = Position::new(0., axis_distance);
            let tick_px = pos_to_canvas(&tick_pos, distance_per_px).y;

            let start = Position::new(tick_start, tick_px);
            let end = Position::new(tick_end, tick_px);
            (start, end)
        }
    };
    draw_line_px(canvas, &start_px, &end_px);
}

fn axis_distance_to_position(axis: &Axis, axis_distance: f32) -> Position {
    let position = match axis {
        Axis::X => Position::new(axis_distance, 0.),
        Axis::Y => Position::new(0., axis_distance),
    };
    position
}
fn draw_ticks_for_axis<T: Renderer>(
    canvas: &mut Canvas<T>,
    axis: &Axis,
    distance_range: (f32, f32),
    interval: i32,
    distance_per_px: f32,
) {
    let (min_distance, max_distance) = distance_range;
    let first_tick = (min_distance / interval as f32).ceil() as i32;
    let last_tick = (max_distance / interval as f32).floor() as i32;

    for i in first_tick..=last_tick {
        let axis_distance = (interval * i) as f32;

        if i == first_tick || i == last_tick {
            let distance_text = format!("{} km", large_number_formatter(axis_distance.into()));
            // println!("writing tick number");
            draw_text_scaled(
                canvas,
                distance_text,
                &axis_distance_to_position(axis, axis_distance),
                25.0,
                distance_per_px,
            );
        }
        draw_tick(canvas, axis, axis_distance, distance_per_px);
    }
}

pub fn draw_tick_marks<T: Renderer>(
    canvas: &mut Canvas<T>,
    x_distance_range: (f32, f32),
    y_distance_range: (f32, f32),
    distance_per_px: f32,
) {
    let (min_distance, max_distance) = x_distance_range;

    let distance = max_distance - min_distance;
    let pow_of_ten = distance.log10().round() as i32;
    let interval = 10_f32.powi(pow_of_ten - 1) as i32;

    for (axis, range) in [(Axis::X, x_distance_range), (Axis::Y, y_distance_range)] {
        draw_ticks_for_axis(canvas, &axis, range, interval, distance_per_px);
    }
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
    // let scaled = scaled_width(canvas, 1.);

    // let total_r = if distance_per_px > 2000. {
    //     fixed_r + scaled
    // } else {
    //     fixed_r
    // };
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
    trajectory: &Vec<Body>,
    ticks_per_graph_point: usize, // number of array elements per graphed point
    distance_per_px: f32,
) {
    let width = scaled_width(canvas, 1.);
    let mut trajectory_path = Path::new();

    let mut trajectory_iter = trajectory.iter();
    let initial_state = trajectory_iter.next();
    match initial_state {
        Some(b) => {
            let canvas_pos = pos_to_canvas(&b.pos, distance_per_px);
            trajectory_path.move_to(canvas_pos.x, canvas_pos.y);
        }
        None => {}
    }
    for b in trajectory_iter.step_by(ticks_per_graph_point) {
        let canvas_pos = pos_to_canvas(&b.pos, distance_per_px);
        trajectory_path.line_to(canvas_pos.x, canvas_pos.y);
    }

    let paint = Paint::color(Color::rgbf(0., 1., 0.)).with_line_width(width);

    canvas.stroke_path(&trajectory_path, &paint);
}

pub fn draw_text<T: Renderer>(
    canvas: &mut Canvas<T>,
    text: String,
    pos: &Position,
    distance_per_px: f32,
) {
    let font_size = (1. / get_scale(canvas)) * 25.;
    draw_text_font(canvas, text, pos, font_size, distance_per_px);
}

pub fn draw_text_scaled<T: Renderer>(
    canvas: &mut Canvas<T>,
    text: String,
    pos: &Position,
    scale_factor: f32,
    distance_per_px: f32,
) {
    let font_size = 1.5 * scale_factor / get_scale(canvas);
    draw_text_font(canvas, text, pos, font_size, distance_per_px);
}

pub fn draw_text_font<T: Renderer>(
    canvas: &mut Canvas<T>,
    text: String,
    pos: &Position,
    font_size: f32,
    distance_per_px: f32,
) {
    let text_paint = Paint::color(Color::white()).with_font_size(font_size);

    let px = pos_to_canvas(pos, distance_per_px);

    canvas
        .fill_text(px.x, px.y, text, &text_paint)
        .expect("failed to write text");
}

pub fn large_number_formatter(num: f64) -> String {
    let abs = num.abs();
    if abs <= 100000. {
        num.to_string()
    } else {
        let pow_of_10 = abs.log10().floor() as i32;

        let decimal = abs / (10_f64.powi(pow_of_10));

        let rounded = round_to_decimals(decimal, 3);
        let with_sign = if num.is_sign_negative() {
            rounded.neg()
        } else {
            rounded
        };

        String::from(format!("{}e{}", with_sign, pow_of_10))
    }
}
