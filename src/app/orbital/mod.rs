use femtovg::{Color, Paint, Path};

use crate::ui::widgets::{CustomSlider, XYInput};

use super::{
    core::{
        draw::scaled_width,
        physics::{Acceleration, Position, Vector, Velocity, G_KM},
    },
    App,
};

pub struct Orbital {
    ui_state: UiState,
    dt: f32,
    started: bool,
    central: Body,
    outer: Body,
    trajectory: Vec<Body>,
}

impl App for Orbital {
    fn run(&mut self) {
        if !self.started {
            return;
        }
        for _ in 1..6000 {
            self.run_euler();
        }
    }

    fn draw(&mut self, canvas: &mut femtovg::Canvas<femtovg::renderer::WGPURenderer>) {
        let mut circles = Path::new();
        let paint = Paint::color(Color::rgbf(0., 1., 0.)).with_line_width(scaled_width(canvas, 1.));

        let central_px = convert_pos_to_canvas(&self.central.pos);
        let outer_px = convert_pos_to_canvas(&self.outer.pos);
        circles.circle(outer_px.x, outer_px.y, scaled_width(canvas, 4.));
        circles.circle(central_px.x, central_px.y, scaled_width(canvas, 10.));

        let sec_per_graph = 100.; // graph a point every 100 seconds
        let ticks_per_graph_point = (sec_per_graph / self.dt).round() as usize;

        let mut filtered = self
            .trajectory
            .iter()
            .enumerate()
            .filter(|(i, _)| i % ticks_per_graph_point == 0)
            .map(|(_, val)| val.pos.clone());

        let mut path = Path::new();
        let initial_pos = filtered.next();
        match initial_pos {
            Some(p) => {
                let canvas_pos = convert_pos_to_canvas(&p);
                path.move_to(canvas_pos.x, canvas_pos.y);
            }
            None => {}
        }
        for pos in filtered {
            let canvas_pos = convert_pos_to_canvas(&pos);
            path.line_to(canvas_pos.x, canvas_pos.y);
        }
        path.line_to(outer_px.x, outer_px.y);

        canvas.fill_path(&circles, &paint);
        canvas.stroke_path(&path, &paint);
    }

    fn enable_ui(&self) -> bool {
        !self.started
    }
    fn ui(&mut self, ctx: &egui::Context) {
        let (kinetic, potential) = self.analyze();

        let panel = egui::SidePanel::left("main-ui-panel")
            .exact_width(self.ui_state.panel_width)
            .resizable(false);
        panel.show(ctx, |ui| {
            if self.started {
                ui.disable();
            }

            ui.label("General");
            ui.add(
                CustomSlider::new(&mut self.dt, 0.01..=10.0)
                    .label("dt:")
                    .full_width(true),
            );

            // ui.label("Central body");
            // ui.add(
            //     CustomSlider::new(&mut self.central.mass, 10000.0..=5.97e24)
            //         .label("M:")
            //         .full_width(true),
            // );
            ui.input(|i| {
                if i.key_pressed(egui::Key::A) {
                    self.reset();
                }
                if !self.started && i.key_pressed(egui::Key::Enter) {
                    self.start();
                }
            });

            let x_range = 0.0..=1000.;
            let y_range = -500.0..=500.;

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

            ui.monospace("Energy (MJ)");
            ui.monospace(format!("Kinetic:    {:+.4e}", kinetic));
            ui.monospace(format!("Potential:  {:+.4e}", potential));
            ui.monospace(format!("Total:      {:+.4e}", kinetic + potential));
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
            dt: 0.1,
            started: false,
            central: Body::earth(),
            outer: Body::outer_low(),
            trajectory: vec![],
        }
    }

    fn start(&mut self) {
        self.started = true;
        self.trajectory.push(self.outer.clone());
    }

    // contains calculations not necessary for the iteration process, only for displaying
    fn analyze(&self) -> (f32, f32) {
        let g = G_KM;

        // Ek = .5mv^2
        let kinetic_mj = 0.5 * self.outer.mass * self.outer.v.mag().powi(2); // MJ

        // Eg = -G * M * m / r
        let grav_potential_kj = -g * self.central.mass * self.outer.mass / self.outer.pos.mag(); // KJ
        let grav_potential_mj = grav_potential_kj * 1e-3; // MJ

        (kinetic_mj, grav_potential_mj)
    }

    // run function contains calculations necessary for the iteration process
    // uses euler method, which is one of the more inaccurate methods
    fn run_euler(&mut self) {
        let dt = self.dt;

        // a = -G * m_central * r_vec / (|r_vec|^3)

        // put central body at (0,0) that way r vector is equal to the position of the outer body
        let r = Position {
            x: self.outer.pos.x,
            y: self.outer.pos.y,
        };

        let a_x = -G_KM * self.central.mass * r.x / r.mag().powi(3); // m/s^2
        let a_x_km = a_x * 1e-3; // km/s^2

        let a_y = -G_KM * self.central.mass * r.y / r.mag().powi(3); // m/s^2
        let a_y_km = a_y * 1e-3; // km/s^2

        let cur_a = Acceleration {
            x: a_x_km,
            y: a_y_km,
        };
        // println!("{:?}", cur_a);

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
        self.trajectory.push(self.outer.clone());
    }

    fn reset(&mut self) {
        match self.trajectory.get(0) {
            Some(initial_body) => {
                self.outer = initial_body.clone();
            }
            None => {
                self.outer = Body::outer_low();
            }
        }
        self.trajectory = vec![];
        self.started = false;
    }
}

#[derive(Default, Clone)]
struct Body {
    pos: Position,
    v: Velocity,
    _a: Acceleration,
    mass: f32,
}
impl Body {
    fn _mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }
    fn outer_low() -> Self {
        Self {
            mass: 400000., // kg
            pos: Position {
                x: 0.,
                y: 400. + 6378., // km, radius of earth = 6378
            },
            v: Velocity { x: 7.8, y: 0. }, // km/s
            ..Default::default()
        }
    }
    fn _outer_med() -> Self {
        Self {
            mass: 5000.,
            pos: Position { x: 0., y: 20000. },
            v: Velocity { x: 3.9, y: 0. },
            ..Default::default()
        }
    }
    fn earth() -> Self {
        Self {
            mass: 5.97e24, // kg
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
