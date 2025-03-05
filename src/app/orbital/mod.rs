use egui::RichText;

use crate::ui::widgets::{CustomSlider, XYInput};

use super::{
    core::{
        draw::{draw_circle_fixed, draw_circle_scaled, draw_line_thru_points},
        physics::{
            circular_velocity, escape_velocity, gravitational_acceleration,
            gravitational_potential_energy, kinetic_energy, symplectic_euler_calc, Position,
            Velocity, R_EARTH_KM,
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
    bodies: Vec<Body>,
    relationships: Vec<(usize, usize)>,
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

    fn draw(&self, canvas: &mut femtovg::Canvas<femtovg::renderer::WGPURenderer>) {
        let central = self.bodies.get(0).unwrap();
        let outer = self.bodies.get(1).unwrap();

        draw_circle_fixed(canvas, &central.pos, central.radius, self.distance_per_px);
        draw_circle_scaled(canvas, &outer.pos, 4., self.distance_per_px);

        let sec_per_graph = 100.; // graph a point every 100 seconds
        let graph_frequency = (sec_per_graph / self.dt).round() as usize;
        // draw a point every X number of ticks

        let points: Vec<Position> = outer.trajectory.iter().map(|b| b.pos).collect();

        draw_line_thru_points(canvas, points, graph_frequency, self.distance_per_px);
    }

    fn ui(&mut self, ctx: &egui::Context) {
        let (kinetic, potential, diff_percent) = self.analyze();

        let panel = egui::SidePanel::left("main-ui-panel")
            .exact_width(self.ui_state.panel_width)
            .resizable(false);
        panel.show(ctx, |ui| {
            ui.add_enabled_ui(!self.started, |ui| {
                ui.label(RichText::new("General").heading());
                ui.add(CustomSlider::new(&mut self.dt, 0.01..=10.0).label("dt:"));
                ui.add(
                    CustomSlider::new(&mut self.num_ticks, 100..=100000).label("ticks per press:"),
                );
                ui.add_space(20.);

                ui.input(|i| {
                    if i.key_pressed(egui::Key::A) {
                        self.reset();
                    }
                    if !self.started && i.key_pressed(egui::Key::Enter) {
                        self.start();
                    }
                });

                let len = self.bodies.len();

                for (i, body) in self.bodies.iter_mut().enumerate() {
                    let x_range = -10000.0..=10000.;
                    let y_range = -10000.0..=10000.;

                    egui::CollapsingHeader::new(
                        RichText::new(format!("Body {}:", i + 1)).heading(),
                    )
                    .show(ui, |ui| {
                        ui.label("Position");
                        ui.add(XYInput::new(
                            &mut body.pos.x,
                            &mut body.pos.y,
                            x_range,
                            y_range,
                        ));
                        ui.label(format!("|r|: {}", body.pos.mag()));

                        if !body.is_fixed {
                            ui.label("Velocity");
                            ui.checkbox(
                                &mut body.lock_to_circular_velocity,
                                "lock to circular velocity",
                            );
                            ui.checkbox(
                                &mut body.lock_to_escape_velocity,
                                "lock to escape velocity",
                            );

                            let vel_lock_enabled =
                                body.lock_to_circular_velocity || body.lock_to_escape_velocity;
                            ui.add_enabled_ui(vel_lock_enabled, |ui| {
                                egui::ComboBox::from_label("Around body").show_index(
                                    ui,
                                    &mut body.selected_vel_lock,
                                    len,
                                    |i| format!("Body {}", i + 1),
                                );
                            });
                            ui.add_enabled_ui(!vel_lock_enabled, |ui| {
                                ui.add(XYInput::new(
                                    &mut body.v.x,
                                    &mut body.v.y,
                                    -50.0..=50.0,
                                    -50.0..=50.0,
                                ));
                            });
                        }

                        ui.label("Mass");
                        ui.add(CustomSlider::new(&mut body.mass, 1.0..=5e10).label("M:"));
                    });
                    ui.add_space(20.);
                }

                self.set_velocities();

                if ui.button("Start").clicked() {
                    self.start();
                }
            });

            let t = self.t();
            let days = t / (60 * 60 * 24) as f32;
            ui.monospace(format!("t: {:.4e} s, {:.2} d", t, days));
            // ui.monospace("Acceleration (km/s^2)");
            // ui.monospace(format!("Ax:    {:+.4e}", cur_a.x));
            // ui.monospace(format!("Ay:    {:+.4e}", cur_a.y));
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

// if the object being pulled is 1000x more massive than the source of the gravity,
// then the gravitational force is negligible
fn is_mass_significant(source_body: &Body, body_under_effect: &Body) -> bool {
    let ratio_threshold = 1000.;
    (body_under_effect.mass / source_body.mass) < ratio_threshold
}

impl Orbital {
    pub fn new() -> Self {
        Self {
            ui_state: UiState::new(),
            dt: 1.,
            num_ticks: 1000,
            distance_per_px: 4.,
            started: false,
            stopped: false,
            initial_e: 0.,
            bodies: vec![Body::earth(), Body::outer_low()],
            relationships: vec![],
        }
    }

    fn t(&self) -> f32 {
        let outer = &self.bodies[1];
        let len = outer.trajectory.len();

        if len > 0 {
            (len - 1) as f32 * self.dt
        } else {
            0.
        }
    }

    fn set_velocities(&mut self) {
        if self.started {
            return;
        }

        let positions: Vec<(Position, f32)> = self.bodies.iter().map(|b| (b.pos, b.mass)).collect();

        for body in self.bodies.iter_mut() {
            if body.lock_to_circular_velocity {
                let (locked_body_pos, locked_body_m) =
                    positions.get(body.selected_vel_lock).unwrap();

                let circ_vel = circular_velocity(*locked_body_pos, *locked_body_m, body.pos);
                body.v = circ_vel;
            } else if body.lock_to_escape_velocity {
                let (locked_body_pos, locked_body_m) =
                    positions.get(body.selected_vel_lock).unwrap();

                let esc_vel = escape_velocity(*locked_body_pos, *locked_body_m, body.pos);
                body.v = esc_vel;
            }
        }
    }

    fn start(&mut self) {
        let relationships_to_calculate: Vec<(usize, usize)> = self
            .bodies
            .iter()
            .enumerate()
            .flat_map(|(source_index, source_body)| {
                let relationships_for_source: Vec<(usize, usize)> = self
                    .bodies
                    .iter()
                    .enumerate()
                    .filter(|(i, body_under_effect)| {
                        *i != source_index && is_mass_significant(source_body, body_under_effect)
                    })
                    .map(|(i, _)| (source_index, i))
                    .collect();

                relationships_for_source
            })
            .collect();

        println!("{:?}", relationships_to_calculate);

        let outer = &mut self.bodies[1];

        self.relationships = relationships_to_calculate;
        self.started = true;
        outer.trajectory.push(outer.store());

        let (_, _, total) = self.current_e();

        self.initial_e = total;
    }

    // contains calculations not necessary for the iteration process, only for displaying
    fn analyze(&self) -> (f32, f32, f32) {
        let (kinetic_mj, grav_potential_mj, total) = self.current_e();

        let diff_percentage = if self.initial_e != 0. {
            (total / self.initial_e) * 100.
        } else {
            0.
        };

        // TODO calc acceleration for each body
        // let cur_a = gravitational_acceleration(self.central.pos, self.outer.pos, self.central.mass);

        // (kinetic_mj, grav_potential_mj, diff_percentage, cur_a)
        (kinetic_mj, grav_potential_mj, diff_percentage)
    }

    fn current_e(&self) -> (f32, f32, f32) {
        let (total_kinetic, total_gravitational) = self
            .bodies
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let body_kinetic_mj = kinetic_energy(b.mass, b.v);

                // loop over all other bodies, so start at the next index
                let body_gravitational_mj = self.bodies[i + 1..].iter().fold(0., |acc, b2| {
                    let grav_potential_mj =
                        gravitational_potential_energy(b.mass, b2.mass, b.pos, b2.pos);

                    acc + grav_potential_mj
                });

                (body_kinetic_mj, body_gravitational_mj)
            })
            .fold((0., 0.), |(acc_k, acc_g), (kinet, grav)| {
                (acc_k + kinet, acc_g + grav)
            });

        let total = total_kinetic + total_gravitational;
        (total_kinetic, total_gravitational, total)
    }

    // run function contains calculations necessary for the iteration process
    fn run_euler(&mut self) {
        let dt = self.dt;

        let central = self.bodies.get(0).unwrap();
        let outer = self.bodies.get(1).unwrap();

        let cur_a = gravitational_acceleration(central.pos, outer.pos, central.mass);

        let cur_v = outer.v;
        let cur_r = outer.pos;
        let (next_r, next_v) = symplectic_euler_calc(cur_r, cur_v, cur_a, dt);

        if next_r.mag() <= central.radius {
            self.stopped = true;
            return;
        }

        let new_outer = Body {
            v: next_v,
            pos: next_r,
            mass: outer.mass,
            radius: outer.radius,
            is_fixed: outer.is_fixed,
            lock_to_circular_velocity: outer.lock_to_circular_velocity,
            lock_to_escape_velocity: outer.lock_to_escape_velocity,
            selected_vel_lock: outer.selected_vel_lock,
            trajectory: vec![],
        };
        self.bodies[1].v = new_outer.v;
        self.bodies[1].pos = new_outer.pos;
        self.bodies[1].trajectory.push(new_outer);
    }

    fn reset(&mut self) {
        let initial_bodies: Vec<Body> = self
            .bodies
            .iter()
            .map(|body| match body.trajectory.get(0) {
                Some(initial_body) => initial_body.clone(),
                None => body.clone(),
            })
            .collect();
        self.bodies = initial_bodies;

        self.started = false;
        self.stopped = false;
    }
}

#[derive(Default, Clone, Debug)]
struct Body {
    pos: Position,
    v: Velocity,
    mass: f32,
    radius: f32,
    trajectory: Vec<Body>,
    is_fixed: bool,
    lock_to_circular_velocity: bool,
    lock_to_escape_velocity: bool,
    selected_vel_lock: usize,
}
impl Body {
    fn _mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    // returns a version of this struct to be used for the trajectory history
    // maybe make this a separate struct?
    fn store(&self) -> Self {
        Self {
            pos: self.pos,
            v: self.v,
            mass: self.mass,
            radius: self.radius,
            is_fixed: self.is_fixed,
            lock_to_circular_velocity: self.lock_to_circular_velocity,
            lock_to_escape_velocity: self.lock_to_escape_velocity,
            selected_vel_lock: self.selected_vel_lock,
            trajectory: vec![],
        }
    }

    // starting conditions for a low earth orbit, modeled after the ISS
    fn outer_low() -> Self {
        let earth_mass = Self::earth().mass;
        let earth_pos = Self::earth().pos;

        let r = 400. + R_EARTH_KM;
        let x = 3000_f32;
        let y = (r.powi(2) - x.powi(2)).sqrt();
        let position = Position::new(x, y);

        Self {
            mass: 400000., // kg
            pos: position,
            // v: escape_velocity(earth_mass, position), // km/s
            v: circular_velocity(earth_pos, earth_mass, position), // km/s
            trajectory: Vec::new(),
            ..Default::default()
        }
    }
    fn _outer_med() -> Self {
        Self {
            mass: 5000.,
            pos: Position::new(5000., 15000.),
            v: Velocity::new(3.9, 0.),
            ..Default::default()
        }
    }
    fn earth() -> Self {
        Self {
            mass: 5.97e24,      // kg
            radius: R_EARTH_KM, // km
            is_fixed: true,
            ..Default::default()
        }
    }
    fn _moon() -> Self {
        let earth_mass = Self::earth().mass;
        let earth_pos = Self::earth().pos;
        let position = Position::new(0., 3.844e5 + R_EARTH_KM);

        Self {
            mass: 7.34e22,
            pos: position,
            v: circular_velocity(earth_pos, earth_mass, position), // km/s
            ..Default::default()
        }
    }
}
