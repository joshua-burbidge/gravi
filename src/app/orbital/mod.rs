use femtovg::{Color, Paint, Path};

use crate::ui::widgets::{CustomSlider, XYInput};

use super::{
    core::{Acceleration, Position, Velocity},
    App,
};

pub struct Orbital {
    ui_state: UiState,
    started: bool,
    central: Body,
    outer: Body,
    trajectory: Vec<Position>,
}

impl App for Orbital {
    fn run(&mut self) {
        self.run_euler();
    }

    fn draw(&mut self, canvas: &mut femtovg::Canvas<femtovg::renderer::WGPURenderer>) {
        let mut path = Path::new();
        let paint = Paint::color(Color::white()).with_line_width(2.);

        let central_px = convert_pos_to_canvas(&self.central.pos);
        let outer_px = convert_pos_to_canvas(&self.outer.pos);
        path.circle(outer_px.x, outer_px.y, 10.);
        path.circle(central_px.x, central_px.y, 25.);

        for p in &self.trajectory {
            let canvas_pos = convert_pos_to_canvas(p);
            path.circle(canvas_pos.x, canvas_pos.y, 3.);
        }

        canvas.fill_path(&path, &paint);
    }

    fn ui(&mut self, ctx: &egui::Context) {
        let panel = egui::SidePanel::left("main-ui-panel")
            .exact_width(self.ui_state.panel_width)
            .resizable(false);
        panel.show(ctx, |ui| {
            if self.started {
                ui.disable();
            }
            ui.label("Central body");
            let x_range = 0.0..=1000.;
            let y_range = -500.0..=500.;
            // ui.add(
            //     CustomSlider::new(&mut self.central.mass, 10000.0..=5.97e24)
            //         .label("M:")
            //         .full_width(true),
            // );

            ui.label("Outer body");
            ui.label("Position");
            ui.add(XYInput::new(
                &mut self.outer.pos.x,
                &mut self.outer.pos.y,
                x_range,
                y_range,
            ));
            ui.label("Velocity");
            ui.add(XYInput::new(
                &mut self.outer.v.x,
                &mut self.outer.v.y,
                0.0..=1000.0,
                0.0..=1.0,
            ));
            ui.add(
                CustomSlider::new(&mut self.outer.mass, 1.0..=5e5)
                    .label("M:")
                    .full_width(true),
            );

            if ui.button("Start").clicked() {
                self.start();
            }
        });
    }
    fn panel_width(&self) -> f32 {
        self.ui_state.panel_width
    }
}

impl Orbital {
    pub fn new() -> Self {
        Self {
            ui_state: UiState::new(),
            started: false,
            central: Body::default().mass(5.97e24),
            outer: Body::outer(),
            trajectory: vec![],
        }
    }

    fn start(&mut self) {
        self.started = true;
        self.trajectory.push(self.outer.pos.clone());
    }

    fn run_euler(&mut self) {
        let dt = 1.;

        // a = -G * m_central * r_vec / (|r_vec|^3)

        // put central body at (0,0) that way r vector is equal to the position of the outer body
        let r = Position {
            x: self.outer.pos.x,
            y: self.outer.pos.y,
        };
        let g = 6.674e-11;
        let a_x = -g * self.central.mass * r.x / r.mag().powi(3);

        let a_y = -g * self.central.mass * r.y / r.mag().powi(3);
        let cur_a = Acceleration { x: a_x, y: a_y };
        println!("{:?}", cur_a);

        // v(t + dt) = v(t) + a(t)*dt
        let next_v = Velocity {
            x: self.outer.v.x + cur_a.x * dt,
            y: self.outer.v.y + cur_a.y * dt,
        };

        // r(t + dt) = r(t) + v(t)*dt
        let next_r = Position {
            x: self.outer.pos.x + self.outer.v.x * dt,
            y: self.outer.pos.y + self.outer.v.y * dt,
        };

        self.outer.v = next_v;
        self.outer.pos = next_r;
        self.trajectory.push(self.outer.pos.clone());
    }
}

#[derive(Default)]
struct Body {
    pos: Position,
    v: Velocity,
    a: Acceleration,
    mass: f32,
}
impl Body {
    fn mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }
    fn outer() -> Self {
        Self {
            mass: 400000.,
            pos: Position { x: 0., y: 400. },
            v: Velocity { x: 7.8, y: 0. },
            ..Default::default()
        }
    }
}
struct UiState {
    panel_width: f32,
}
impl UiState {
    fn new() -> Self {
        Self { panel_width: 300. }
    }
}

fn convert_pos_to_canvas(pos: &Position) -> Position {
    Position {
        x: pos.x,
        y: -pos.y,
    }
}
