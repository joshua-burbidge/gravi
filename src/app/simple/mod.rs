use femtovg::{renderer::WGPURenderer, Canvas, Color, Paint, Path};

use super::{
    core::{new_position, new_vel, Acceleration, Position, Velocity},
    App,
};

pub struct ConstAcceleration {
    pub ui_state: UiState,
    started: bool,
    hist: Vec<Position>,
    v: Velocity,
    a: Acceleration,
    t_per_tick: f32,
}
impl App for ConstAcceleration {
    fn run(&mut self) {
        if self.started {
            let new_pos = new_position(self.current(), &self.v, &self.a, self.t_per_tick);
            let new_vel = new_vel(&self.v, &self.a, self.t_per_tick);

            self.hist.push(new_pos);
            self.v = new_vel;
        }
    }

    fn draw(&mut self, canvas: &mut Canvas<WGPURenderer>) {
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

    fn ui(&mut self, ctx: &egui::Context) {
        let panel = egui::SidePanel::left("main-ui-panel")
            .exact_width(self.ui_state.panel_width)
            .resizable(false);

        panel.show(ctx, |ui| {
            ui.style_mut().spacing.slider_width = ui.available_width() - 50.0;

            if self.started {
                ui.disable();
            }
            ui.label("Choose starting position");
            ui.add(egui::Slider::new(&mut self.ui_state.start_pos, 0..=1000));
            ui.label("Choose starting velocity");
            ui.horizontal(|ui| {
                ui.label("X:");
                ui.style_mut().spacing.slider_width = ui.available_width() - 50.0;
                ui.add(egui::Slider::new(&mut self.ui_state.vx, 0.0..=100.));
            });
            ui.horizontal(|ui| {
                ui.label("Y:");
                ui.style_mut().spacing.slider_width = ui.available_width() - 50.0;
                ui.add(egui::Slider::new(&mut self.ui_state.vy, 0.0..=100.));
            });
            ui.label("Apply additional constant y acceleration");
            ui.add(egui::Slider::new(&mut self.ui_state.accel, -99.9..=99.9));
            if ui.button("Start").clicked() {
                self.start();
            }
        });
    }

    fn panel_width(&self) -> f32 {
        self.ui_state.panel_width
    }
}

pub struct UiState {
    pub panel_width: f32,
    pub start_pos: i32,
    pub accel: f32,
    pub vx: f32,
    pub vy: f32,
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            panel_width: 300.,
            start_pos: 0,
            accel: 0.,
            vx: 0.,
            vy: 0.,
        }
    }
}

impl ConstAcceleration {
    pub fn new() -> Self {
        ConstAcceleration {
            ui_state: UiState::default(),
            started: false,
            hist: vec![],
            v: Velocity { x: 0., y: 0. },
            a: Acceleration { x: 0., y: -9.8 },
            t_per_tick: 0.25,
        }
    }

    pub fn start(&mut self) {
        let start_pos = Position::new(self.ui_state.start_pos as f32, 0.);
        self.hist = vec![start_pos];
        self.started = true;

        self.a = Acceleration {
            x: self.a.x,
            y: self.a.y + self.ui_state.accel,
        };
        self.v = Velocity {
            x: self.ui_state.vx,
            y: self.ui_state.vy,
        }
    }

    fn current(&self) -> &Position {
        let index = self.hist.len() - 1;
        &self.hist[index]
    }
}

fn convert_pos_to_canvas(pos: &Position) -> Position {
    Position {
        x: pos.x,
        y: -pos.y,
    }
}
