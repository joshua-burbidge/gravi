use egui::RichText;
use std::collections::HashMap;

use crate::ui::widgets::{CustomSlider, XYInput};

use super::{
    core::{
        draw::{draw_circle_fixed, draw_circle_scaled, draw_line_thru_points},
        physics::{
            circular_velocity, escape_velocity, gravitational_acceleration,
            gravitational_potential_energy, kinetic_energy, symplectic_euler_calc, Acceleration,
            Position, Velocity, R_EARTH_KM, R_MOON_KM,
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
    relationships: HashMap<usize, Vec<usize>>,
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
        let sec_per_graph = 100.; // graph a point every 100 seconds
        let graph_frequency = (sec_per_graph / self.dt).ceil() as usize;

        for b in self.bodies.iter() {
            if b.radius == 0. {
                draw_circle_scaled(canvas, &b.pos, 4., self.distance_per_px);
            } else {
                draw_circle_fixed(canvas, &b.pos, b.radius, self.distance_per_px);
            }

            let points: Vec<Position> = b.trajectory.iter().map(|b| b.pos).collect();

            draw_line_thru_points(canvas, points, graph_frequency, self.distance_per_px);
        }
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

                        ui.monospace("Acceleration (km/s^2)");
                        ui.monospace(format!("Ax:    {:+.4e}", body.computed_a.x));
                        ui.monospace(format!("Ay:    {:+.4e}", body.computed_a.y));
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
            bodies: vec![Body::earth(), Body::outer_low(), Body::_moon()],
            relationships: HashMap::new(),
        }
    }

    fn t(&self) -> f32 {
        let body = self.bodies.first();
        let len = match body {
            Some(b) => b.trajectory.len(),
            None => 0,
        };

        if len > 0 {
            (len - 1) as f32 * self.dt
        } else {
            0.
        }
    }

    // Set circular or orbital velocity for any body that is locked to one of those.
    // Only applies when setting initial conditions before starting.
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
        // map of a body to the list of bodies that have a gravitational effect on it
        let mut map_body_to_sources: HashMap<usize, Vec<usize>> = HashMap::new();

        for (effected_i, affected_body) in self.bodies.iter().enumerate() {
            let significant_sources: Vec<usize> = self
                .bodies
                .iter()
                .enumerate()
                .filter(|(source_i, source_body)| {
                    *source_i != effected_i && is_mass_significant(source_body, affected_body)
                })
                .map(|(source_i, _)| (source_i))
                .collect();

            map_body_to_sources.insert(effected_i, significant_sources);
        }

        println!(
            "significant gravitational relationships: {:?}",
            map_body_to_sources
        );

        self.relationships = map_body_to_sources;
        self.started = true;

        for b in self.bodies.iter_mut() {
            b.trajectory.push(b.new_history_entry());
        }

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

        for (affected_i, sources) in self.relationships.iter() {
            let affected = self.bodies.get(*affected_i).unwrap();

            let total_a_for_body = sources
                .iter()
                .map(|source_i| {
                    let source = self.bodies.get(*source_i).unwrap();
                    let a_from_source =
                        gravitational_acceleration(source.pos, affected.pos, source.mass);

                    // if *affected_i == 1 {
                    //     println!("{:?}", (affected_i, source_i, a_from_source));
                    // }
                    a_from_source
                })
                .fold(Acceleration::new(0., 0.), |acc, a| acc.add(a));

            // if *affected_i == 1 {
            //     println!("{:?}", total_a_for_body);
            // }

            let cur_v = affected.v;
            let cur_r = affected.pos;
            let (next_r, next_v) = symplectic_euler_calc(cur_r, cur_v, total_a_for_body, dt);

            // if next_r.mag() <= central.radius {
            //     self.stopped = true;
            //     return;
            // }

            self.bodies[*affected_i].update(next_r, next_v, total_a_for_body);
        }
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
    computed_a: Acceleration,
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
    fn new_history_entry(&self) -> Self {
        Self {
            pos: self.pos,
            v: self.v,
            mass: self.mass,
            radius: self.radius,
            computed_a: self.computed_a,
            is_fixed: self.is_fixed,
            lock_to_circular_velocity: self.lock_to_circular_velocity,
            lock_to_escape_velocity: self.lock_to_escape_velocity,
            selected_vel_lock: self.selected_vel_lock,
            trajectory: vec![],
        }
    }

    fn update(&mut self, new_pos: Position, new_vel: Velocity, new_acc: Acceleration) -> &mut Self {
        self.pos = new_pos;
        self.v = new_vel;
        self.computed_a = new_acc;
        self.trajectory.push(self.new_history_entry());

        self
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
            radius: R_MOON_KM,
            pos: position,
            v: circular_velocity(earth_pos, earth_mass, position), // km/s
            ..Default::default()
        }
    }
}
