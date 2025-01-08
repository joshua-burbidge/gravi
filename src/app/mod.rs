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
    fn new(x: f32, y: f32) -> Self {
        Position { x, y }
    }
}
struct PositionHistory {
    history: Vec<Position>,
}
impl PositionHistory {
    fn new() -> Self {
        PositionHistory { history: vec![] }
    }
    fn from_start(start: Position) -> Self {
        PositionHistory {
            history: vec![start],
        }
    }
}

pub struct AppState {
    pos: Position,
    hist: Vec<Position>,
}
impl AppState {
    pub fn new() -> Self {
        let starting_pos = Position { x: 0., y: -100. };
        AppState {
            pos: starting_pos.clone(),
            hist: vec![starting_pos],
        }
    }

    pub fn run(&mut self, canvas: &mut Canvas<WGPURenderer>, next_tick: bool) {
        let speed = Position { x: 10., y: 2. };

        if next_tick {
            let new_pos = Position::new(self.pos.x + speed.x, self.pos.y + speed.y);
            self.pos = new_pos.clone();
            self.hist.push(new_pos);
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
