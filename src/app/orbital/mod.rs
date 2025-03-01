use crate::ui::widgets::{CustomSlider, XYInput};

use super::{
    core::{
        draw::{draw_circle_fixed, draw_circle_scaled, draw_line_thru_points},
        physics::{
            circular_velocity, escape_velocity, gravitational_acceleration, symplectic_euler_calc,
            Acceleration, Position, Velocity, G_KM, R_EARTH_KM,
        },
    },
    App,
};

pub struct Orbital {
    ui_state: UiState,
    dt: f32,
    num_ticks: i32,
    distance_per_px: f32,
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
        for _ in 0..self.num_ticks {
            self.run_euler();
        }
    }

    fn draw(&mut self, canvas: &mut femtovg::Canvas<femtovg::renderer::WGPURenderer>) {
        draw_circle_fixed(
            canvas,
            &self.central.pos,
            self.central.radius,
            self.distance_per_px,
        );
        draw_circle_scaled(canvas, &self.outer.pos, 4., self.distance_per_px);

        let sec_per_graph = 100.; // graph a point every 100 seconds
        let graph_frequency = (sec_per_graph / self.dt).round() as usize;
        // draw a point every X number of ticks

        let points: Vec<Position> = self.trajectory.iter().map(|b| b.pos).collect();

        draw_line_thru_points(canvas, points, graph_frequency, self.distance_per_px);
    }

    fn ui(&mut self, ctx: &egui::Context) {
        let (kinetic, potential, diff_percent, cur_a) = self.analyze();

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

                let x_range = -10000.0..=10000.;
                let y_range = -10000.0..=10000.;

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
                ui.checkbox(
                    &mut self.ui_state.lock_circular_velocity,
                    "lock to circular velocity",
                );
                ui.checkbox(
                    &mut self.ui_state.lock_escape_velocity,
                    "lock to escape velocity",
                );

                let enable_vel_inputs =
                    !self.ui_state.lock_circular_velocity && !self.ui_state.lock_escape_velocity;
                ui.add_enabled_ui(enable_vel_inputs, |ui| {
                    if !self.started && self.ui_state.lock_circular_velocity {
                        self.outer.v = circular_velocity(self.central.mass, self.outer.pos);
                    }
                    if !self.started && self.ui_state.lock_escape_velocity {
                        self.outer.v = escape_velocity(self.central.mass, self.outer.pos);
                    }
                    ui.add(XYInput::new(
                        &mut self.outer.v.x,
                        &mut self.outer.v.y,
                        -50.0..=50.0,
                        -50.0..=50.0,
                    ));
                });
                ui.add(
                    CustomSlider::new(&mut self.outer.mass, 1.0..=5e10)
                        .label("M:")
                        .full_width(true),
                );

                if ui.button("Start").clicked() {
                    self.start();
                }
            });

            let t = self.t();
            let days = t / (60 * 60 * 24) as f32;
            ui.monospace(format!("t: {:.4e} s, {:.2} d", t, days));
            ui.monospace("Acceleration (km/s^2)");
            ui.monospace(format!("Ax:    {:+.4e}", cur_a.x));
            ui.monospace(format!("Ay:    {:+.4e}", cur_a.y));
            ui.monospace("Energy (MJ)");
            ui.monospace(format!("Kinetic:    {:+.4e}", kinetic));
            ui.monospace(format!("Potential:  {:+.4e}", potential));
            ui.monospace(format!("Total:      {:+.4e}", kinetic + potential));
            ui.monospace(format!("Initial:    {:+.4e}", self.initial_e));
            ui.monospace(format!("Diff:        {:.2}%", diff_percent));
            ui.monospace(format!("Diff per t:  {:.2e}%", (100. - diff_percent) / t));
        });
    }
    fn panel_width(&self) -> f32 {
        self.ui_state.panel_width
    }
}

#[derive(Default)]
struct UiState {
    panel_width: f32,
    lock_circular_velocity: bool,
    lock_escape_velocity: bool,
}
impl UiState {
    fn new() -> Self {
        Self {
            panel_width: 300.,
            ..Default::default()
        }
    }
}

impl Orbital {
    pub fn new() -> Self {
        Self {
            ui_state: UiState::new(),
            dt: 10.,
            num_ticks: 100000,
            distance_per_px: 4.,
            started: false,
            stopped: false,
            initial_e: 0.,
            central: Body::earth(),
            outer: Body::outer_low(),
            // outer: Body::moon(),
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
        self.trajectory.push(self.outer);

        let (_, _, total) = self.current_e();

        self.initial_e = total;
    }

    // contains calculations not necessary for the iteration process, only for displaying
    fn analyze(&self) -> (f32, f32, f32, Acceleration) {
        let (kinetic_mj, grav_potential_mj, total) = self.current_e();

        let diff_percentage = if self.initial_e != 0. {
            (total / self.initial_e) * 100.
        } else {
            0.
        };

        let cur_a = gravitational_acceleration(self.central.pos, self.outer.pos, self.central.mass);

        (kinetic_mj, grav_potential_mj, diff_percentage, cur_a)
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
    fn run_euler(&mut self) {
        let dt = self.dt;

        let cur_a = gravitational_acceleration(self.central.pos, self.outer.pos, self.central.mass);

        let cur_v = self.outer.v;
        let cur_r = self.outer.pos;
        let (next_r, next_v) = symplectic_euler_calc(cur_r, cur_v, cur_a, dt);

        if next_r.mag() <= self.central.radius {
            self.stopped = true;
            return;
        }

        self.outer.v = next_v;
        self.outer.pos = next_r;
        self.trajectory.push(self.outer);
    }

    fn reset(&mut self) {
        match self.trajectory.get(0) {
            Some(initial_body) => {
                self.outer = *initial_body;
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

#[derive(Default, Copy, Clone)]
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

    // starting conditions for a low earth orbit, modeled after the ISS
    fn outer_low() -> Self {
        let earth_mass = Self::earth().mass;

        let r = 400. + R_EARTH_KM;
        let x = 3000_f32;
        let y = (r.powi(2) - x.powi(2)).sqrt();
        let position = Position::new(x, y);

        Self {
            mass: 400000., // kg
            pos: position,
            // v: escape_velocity(earth_mass, position), // km/s
            v: circular_velocity(earth_mass, position), // km/s
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
            mass: 5.97e24,      // kg
            radius: R_EARTH_KM, // km
            ..Default::default()
        }
    }
    fn moon() -> Self {
        let earth_mass = Self::earth().mass;
        let position = Position::new(0., 3.844e5 + R_EARTH_KM);

        Self {
            mass: 7.34e22,
            pos: position,
            v: circular_velocity(earth_mass, position), // km/s
            ..Default::default()
        }
    }
}
