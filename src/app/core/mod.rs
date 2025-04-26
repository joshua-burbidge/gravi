pub mod draw;
pub mod physics;

pub fn midpoint(a: f32, b: f32) -> f32 {
    (a + b) / 2.
}

pub fn eq_tolerance(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() < tol
}
