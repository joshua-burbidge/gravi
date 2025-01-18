#[derive(Clone, Debug, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Position { x, y }
    }
    pub fn mag(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Debug, Default)]
pub struct Acceleration {
    pub x: f32,
    pub y: f32,
}

// position after one tick given constant acceleration
pub fn new_position(p: &Position, v: &Velocity, a: &Acceleration, t: f32) -> Position {
    // px + vx t + 1/2 ax t^2
    Position {
        x: p.x + v.x * t + 0.5 * a.x * t.powi(2),
        y: p.y + v.y * t + 0.5 * a.x * t.powi(2),
    }
}

pub fn new_vel(v: &Velocity, a: &Acceleration, t: f32) -> Velocity {
    // vx + ax t
    Velocity {
        x: v.x + a.x * t,
        y: v.y + a.y * t,
    }
}

pub fn midpoint(a: f32, b: f32) -> f32 {
    (a + b) / 2.
}

pub fn eq_tolerance(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() < tol
}
