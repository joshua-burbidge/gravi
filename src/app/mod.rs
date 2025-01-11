use femtovg::{renderer::WGPURenderer, Canvas, Color, Paint, Path};
use ui::UiState;

mod ui;

// this module is for the main app functionality

#[derive(Clone)]
struct Position {
    x: f32,
    y: f32,
}
impl Position {
    fn new(x: f32, y: f32) -> Self {
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

pub struct App {
    pub ui_state: UiState,
    started: bool,
    hist: Vec<Position>,
    v: Velocity,
    a: Accel,
    t_per_tick: f32,
}
impl App {
    pub fn new() -> Self {
        App {
            ui_state: UiState::default(),
            started: false,
            hist: vec![],
            v: Velocity { x: 15., y: 60. },
            a: Accel { x: 0., y: -9.8 },
            t_per_tick: 0.25,
        }
    }

    fn current(&self) -> &Position {
        let index = self.hist.len() - 1;
        &self.hist[index]
    }

    pub fn run(&mut self) {
        if self.started {
            let new_pos = new_position(self.current(), &self.v, &self.a, self.t_per_tick);
            let new_vel = new_vel(&self.v, &self.a, self.t_per_tick);

            self.hist.push(new_pos);
            self.v = new_vel;
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<WGPURenderer>) {
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

        dots.circle(self.ui_state.start_pos as f32, 0., 3.);

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
fn new_position(p: &Position, v: &Velocity, a: &Accel, t: f32) -> Position {
    // px + vx t + 1/2 ax t^2
    Position {
        x: p.x + v.x * t + 0.5 * a.x * t.powi(2),
        y: p.y + v.y * t + 0.5 * a.x * t.powi(2),
    }
}

fn new_vel(v: &Velocity, a: &Accel, t: f32) -> Velocity {
    // vx + ax t
    Velocity {
        x: v.x + a.x * t,
        y: v.y + a.y * t,
    }
}
