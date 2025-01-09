use femtovg::{renderer::WGPURenderer, Canvas, Color, Paint, Path};

// this module is for the main app functionality

// process:
// 1. if (next_tick) run one tick
// 2. draw

#[derive(Clone)]
struct Position {
    x: f32,
    y: f32,
}
impl Position {
    fn _new(x: f32, y: f32) -> Self {
        Position { x, y }
    }
}

struct Velocity {
    x: f32,
    y: f32,
}
struct Accel {
    x: f32,
    y: f32,
}

pub struct AppState {
    hist: Vec<Position>,
    v: Velocity,
    a: Accel,
}
impl AppState {
    pub fn new() -> Self {
        let starting_pos = Position { x: 0., y: 0. };
        AppState {
            hist: vec![starting_pos],
            v: Velocity { x: 10., y: 50. },
            a: Accel { x: 0., y: -9.8 },
        }
    }

    fn current(&self) -> &Position {
        let index = self.hist.len() - 1;
        &self.hist[index]
    }

    pub fn run(&mut self, canvas: &mut Canvas<WGPURenderer>, next_tick: bool) {
        if next_tick {
            let new_pos = new_position(self.current(), &self.v, &self.a);
            let new_vel = new_vel(&self.v, &self.a);

            self.hist.push(new_pos);
            self.v = new_vel;
        }

        self.draw_app(canvas);
    }

    pub fn draw_app(&mut self, canvas: &mut Canvas<WGPURenderer>) {
        let mut path = Path::new();
        path.move_to(-10000., 0.);
        path.line_to(10000., 0.);

        path.move_to(0., -10000.);
        path.line_to(0., 10000.);
        canvas.stroke_path(&path, &Paint::color(Color::white()));

        let mut dots = Path::new();
        let history = &self.hist;

        for p in history {
            let canvas_pos = convert_pos_to_canvas(p);
            dots.circle(canvas_pos.x, canvas_pos.y, 3.);
        }

        canvas.fill_path(&dots, &Paint::color(Color::white()));
    }
}

fn convert_pos_to_canvas(pos: &Position) -> Position {
    Position {
        x: pos.x,
        y: -pos.y,
    }
}

// position after one tick given constant acceleration
fn new_position(p: &Position, v: &Velocity, a: &Accel) -> Position {
    // px + vx t + 1/2 ax t^2
    Position {
        x: p.x + v.x + 0.5 * a.x,
        y: p.y + v.y + 0.5 * a.x,
    }
}

fn new_vel(v: &Velocity, a: &Accel) -> Velocity {
    // vx + ax t
    Velocity {
        x: v.x + a.x,
        y: v.y + a.y,
    }
}
