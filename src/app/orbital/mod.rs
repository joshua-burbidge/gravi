use femtovg::{Color, Paint, Path};

use crate::ui::widgets::{CustomSlider, XYInput};

use super::{
    core::{
        draw::scaled_width,
        physics::{Acceleration, Position, Velocity, G_KM},
    },
    App,
};

pub struct Orbital {
    ui_state: UiState,
    dt: f32,
    num_ticks: i32,
    started: bool,
    stopped: bool,
    initial_e: f32,
    central: Body,
    outer: Body,
    trajectory: Vec<Body>,
}

impl App for Orbital {
    fn run(&mut self) {
        if !self.started || self.stopped {
            return;
        }
        for _ in 1..self.num_ticks {
            self.run_euler();
        }
    }

    fn draw(&mut self, canvas: &mut femtovg::Canvas<femtovg::renderer::WGPURenderer>) {
        let mut circles = Path::new();

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

        let mut line_path = Path::new();
        let initial_pos = filtered.next();
        match initial_pos {
            Some(p) => {
                let canvas_pos = convert_pos_to_canvas(&p);
                line_path.move_to(canvas_pos.x, canvas_pos.y);
            }
            None => {}
        }
        for pos in filtered {
            let canvas_pos = convert_pos_to_canvas(&pos);
            line_path.line_to(canvas_pos.x, canvas_pos.y);
        }
        line_path.line_to(outer_px.x, outer_px.y);

        let mut radius_path = Path::new();
        radius_path.circle(central_px.x, central_px.y, self.central.radius);

        let paint = Paint::color(Color::rgbf(0., 1., 0.)).with_line_width(scaled_width(canvas, 1.));

        canvas.fill_path(&circles, &paint);
        canvas.stroke_path(&line_path, &paint);
        canvas.stroke_path(&radius_path, &paint);
    }

    fn enable_ui(&self) -> bool {
        !self.started
    }
    fn ui(&mut self, ctx: &egui::Context) {
        let (kinetic, potential, diff) = self.analyze();

        let panel = egui::SidePanel::left("main-ui-panel")
            .exact_width(self.ui_state.panel_width)
            .resizable(false);
        panel.show(ctx, |ui| {
            ui.add_enabled_ui(!self.started, |ui| {
                ui.label("General");
                ui.add(CustomSlider::new(&mut self.dt, 0.01..=10.0).label("dt:"));
                ui.add(
                    CustomSlider::new(&mut self.num_ticks, 100..=100000).label("ticks per press:"),
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
                ui.label(format!("|r|: {}", self.outer.pos.mag()));
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

            let t = self.t();
            let diff_perc = diff * 100.;
            ui.monospace(format!("t: {}", t));
            ui.monospace("Energy (MJ)");
            ui.monospace(format!("Kinetic:    {:+.4e}", kinetic));
            ui.monospace(format!("Potential:  {:+.4e}", potential));
            ui.monospace(format!("Total:      {:+.4e}", kinetic + potential));
            ui.monospace(format!("Initial:    {:+.4e}", self.initial_e));
            ui.monospace(format!("Diff:        {:.2}%", diff_perc));
            ui.monospace(format!("Diff per t:  {:.2e}%", (100. - diff_perc) / t));
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
            num_ticks: 10000,
            started: false,
            stopped: false,
            initial_e: 0.,
            central: Body::earth(),
            outer: Body::outer_low(),
            trajectory: vec![],
        }
    }

    fn t(&self) -> f32 {
        let len = self.trajectory.len();

        if len > 0 {
            (len - 1) as f32 * self.dt
        } else {
            0.
        }
    }

    fn start(&mut self) {
        self.started = true;
        self.trajectory.push(self.outer.clone());

        let (_, _, total) = self.current_e();

        self.initial_e = total;
    }

    // contains calculations not necessary for the iteration process, only for displaying
    fn analyze(&self) -> (f32, f32, f32) {
        let (kinetic_mj, grav_potential_mj, total) = self.current_e();

        let diff = if self.initial_e != 0. {
            total / self.initial_e
        } else {
            0.
        };

        (kinetic_mj, grav_potential_mj, diff)
    }

    fn current_e(&self) -> (f32, f32, f32) {
        let g = G_KM;

        // Ek = .5mv^2
        let kinetic_mj = 0.5 * self.outer.mass * self.outer.v.mag().powi(2); // MJ

        // Eg = -G * M * m / r
        let grav_potential_kj = -g * self.central.mass * self.outer.mass / self.outer.pos.mag(); // KJ
        let grav_potential_mj = grav_potential_kj * 1e-3; // MJ
        let total = kinetic_mj + grav_potential_mj;

        (kinetic_mj, grav_potential_mj, total)
    }

    // run function contains calculations necessary for the iteration process
    // uses euler method, which is one of the more inaccurate methods
    fn run_euler(&mut self) {
        let dt = self.dt;

        // a = -G * m_central * r_vec / (|r_vec|^3)

        // put central body at (0,0) that way r vector is equal to the position of the outer body
        let r = self.outer.pos.clone();

        let a_x = -G_KM * self.central.mass * r.x / r.mag().powi(3); // m/s^2
        let a_x_km = a_x * 1e-3; // km/s^2

        let a_y = -G_KM * self.central.mass * r.y / r.mag().powi(3); // m/s^2
        let a_y_km = a_y * 1e-3; // km/s^2

        let cur_a = Acceleration::new(a_x_km, a_y_km);

        // v(t + dt) = v(t) + a(t)*dt
        let next_v = self.outer.v.update(&cur_a, dt);

        // r(t + dt) = r(t) + v(t)*dt
        let next_r = self.outer.pos.update_const_v(&self.outer.v, dt);

        if next_r.mag() <= self.central.radius {
            self.stopped = true;
            return;
        }

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
        self.stopped = false;
    }
}

#[derive(Default, Clone)]
struct Body {
    pos: Position,
    v: Velocity,
    mass: f32,
    radius: f32,
}
impl Body {
    fn _mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }
    fn outer_low() -> Self {
        Self {
            mass: 400000., // kg
            pos: Position::new(
                0.,
                400. + 6378., // km, radius of earth = 6378
            ),
            v: Velocity::new(7.8, 0.), // km/s
            ..Default::default()
        }
    }
    fn _outer_med() -> Self {
        Self {
            mass: 5000.,
            pos: Position::new(0., 20000.),
            v: Velocity::new(3.9, 0.),
            ..Default::default()
        }
    }
    fn earth() -> Self {
        Self {
            mass: 5.97e24, // kg
            radius: 6378., // km
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
    Position::new(pos.x, -pos.y)
}
