use femtovg::{renderer::WGPURenderer, Canvas, Color, Paint, Path};

// process:
// 1. if (next_tick) run one tick
// 2. draw

pub struct AppState {
    pub pos: (f32, f32),
}
impl AppState {
    pub fn new() -> Self {
        AppState { pos: (0., 100.) }
    }

    pub fn draw_app(&mut self, canvas: &mut Canvas<WGPURenderer>, next_tick: bool) {
        let mut path = Path::new();
        path.move_to(0., 0.);
        path.line_to(300., 300.);
        canvas.stroke_path(&path, &Paint::color(Color::white()));

        let speed = 10.;

        if next_tick {
            let new_pos = (self.pos.0 + speed, self.pos.1);
            self.pos = new_pos;
        }

        let mut dots = Path::new();
        dots.move_to(self.pos.0, self.pos.1);
        dots.circle(self.pos.0, self.pos.1, 10.);
        canvas.fill_path(&dots, &Paint::color(Color::white()));
    }
}
