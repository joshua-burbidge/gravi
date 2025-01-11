#[derive(Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Position { x, y }
    }
}

pub struct Velocity {
    pub x: f32,
    pub y: f32,
}
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
