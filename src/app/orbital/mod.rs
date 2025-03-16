mod body;
mod ui;

use std::collections::HashMap;

use body::{is_mass_significant, Body, Preset};

use super::{
    core::{
        draw::{
            draw_barycenter, draw_circle_by_radius, draw_circle_scaled, draw_line_thru_points,
            draw_text,
        },
        physics::{
            barycenter, circ_velocity_barycenter, escape_velocity_barycenter,
            gravitational_acceleration, gravitational_potential_energy, kinetic_energy,
            symplectic_euler_calc, Acceleration, Position,
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
    bodies: Vec<Body>,
    relationships: HashMap<usize, Vec<usize>>,
    presets: Vec<Preset>,
    analysis: Analysis,
}

impl App for Orbital {
    fn run(&mut self) {
        if !self.started || self.stopped {
            return;
        }
        for _ in 0..self.num_ticks {
            self.run_euler();
        }
        self.analyze();
    }

    fn draw(&self, canvas: &mut femtovg::Canvas<femtovg::renderer::WGPURenderer>) {
        let sec_per_graph = 100.; // graph a point every 100 seconds
        let graph_frequency = (sec_per_graph / self.dt).ceil() as usize;

        for b in self.bodies.iter() {
            if b.radius == 0. {
                draw_circle_scaled(canvas, &b.pos, 10., self.distance_per_px);
            } else {
                draw_circle_by_radius(canvas, &b.pos, b.radius, self.distance_per_px);
            }

            let points: Vec<Position> = b.trajectory.iter().map(|b| b.pos).collect();

            draw_line_thru_points(canvas, points, graph_frequency, self.distance_per_px);

            draw_text(canvas, b.name.clone(), &b.pos, self.distance_per_px);
        }

        for bary in &self.analysis.barycenters {
            draw_barycenter(canvas, &bary, 5., self.distance_per_px);
        }
    }

    fn ui(&mut self, ctx: &egui::Context) {
        ui::ui(self, ctx);
        ui::controls_panel(self, ctx);
        self.analyze();
    }
    fn panel_width(&self) -> f32 {
        self.ui_state.panel_width
    }
}

impl Orbital {
    pub fn new() -> Self {
        Self {
            ui_state: UiState::new(),
            dt: 1.,
            num_ticks: 1000,
            distance_per_px: 150.,
            started: false,
            stopped: false,
            bodies: vec![Body::earth()],
            relationships: HashMap::new(),
            presets: Preset::defaults(),
            analysis: Analysis::default(),
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

    fn load_preset(&mut self, preset_num: usize) {
        let preset = self.presets.get(preset_num);

        match preset {
            Some(preset) => {
                self.bodies = preset.bodies.clone();
                self.distance_per_px = preset.distance_per_px as f32;
            }
            None => {}
        };
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

                let (circ_vel, _) =
                    circ_velocity_barycenter(body.mass, body.pos, *locked_body_m, *locked_body_pos);
                body.v = circ_vel;
            } else if body.lock_to_escape_velocity {
                let (locked_body_pos, locked_body_m) =
                    positions.get(body.selected_vel_lock).unwrap();

                let esc_vel = escape_velocity_barycenter(
                    body.mass,
                    body.pos,
                    *locked_body_m,
                    *locked_body_pos,
                );
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

        self.analysis = self.analysis.initialize(self);
    }

    // run function contains calculations necessary for the iteration process
    fn run_euler(&mut self) {
        if self.stopped {
            return;
        }

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

            self.bodies[*affected_i].update(next_r, next_v, total_a_for_body);
        }

        self.check_collisions();
    }

    fn check_collisions(&mut self) -> bool {
        for (i, b) in self.bodies.iter().enumerate() {
            for b2 in self.bodies[i + 1..].iter() {
                let distance_between = b.pos.minus(b2.pos).mag();

                let is_collided = distance_between <= (b.radius + b2.radius);

                if is_collided {
                    self.stopped = true;
                    return true;
                }
            }
        }
        false
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

    fn analyze(&mut self) {
        self.analysis = self.analysis.analyze(self);
    }
}

#[derive(Default)]
struct UiState {
    panel_width: f32,
}
impl UiState {
    fn new() -> Self {
        Self {
            panel_width: 300.,
            ..Default::default()
        }
    }
}

#[derive(Default)]
struct Analysis {
    initial_e: f32,
    kinetic_e: f32,
    gravitational_e: f32,
    diff_percentage: f32,
    barycenters: Vec<Position>,
}

// contains calculations not necessary for the iteration process, only for displaying
impl Analysis {
    fn analyze(&self, app: &Orbital) -> Analysis {
        let (kinetic_mj, grav_potential_mj, total) = self.current_e(app);
        let diff_percentage = if self.initial_e != 0. {
            (total / self.initial_e) * 100.
        } else {
            0.
        };

        let barycenters = self.barycenters(app);

        Analysis {
            kinetic_e: kinetic_mj,
            gravitational_e: grav_potential_mj,
            diff_percentage,
            initial_e: self.initial_e,
            barycenters,
        }
    }

    fn current_e(&self, app: &Orbital) -> (f32, f32, f32) {
        let (total_kinetic, total_gravitational) = app
            .bodies
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let body_kinetic_mj = kinetic_energy(b.mass, b.v);

                // loop over all other bodies, so start at the next index
                let body_gravitational_mj = app.bodies[i + 1..].iter().fold(0., |acc, b2| {
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

    fn barycenters(&self, app: &Orbital) -> Vec<Position> {
        let mut barycenters = vec![];

        for (affected_i, sources) in app.relationships.iter() {
            let affected = app.bodies.get(*affected_i).unwrap();

            for source_i in sources.iter() {
                let source = app.bodies.get(*source_i).unwrap();

                let barycenter = barycenter(affected.mass, source.mass, affected.pos, source.pos);
                barycenters.push(barycenter);
            }
        }

        barycenters
    }

    fn initialize(&self, app: &Orbital) -> Analysis {
        let mut initial_analysis = self.analyze(app);
        let total_e = initial_analysis.kinetic_e + initial_analysis.gravitational_e;

        initial_analysis.initial_e = total_e;

        initial_analysis
    }
}
