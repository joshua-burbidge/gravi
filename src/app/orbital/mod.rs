pub mod body;
mod ui;

use body::{is_mass_significant, Body, Preset};
use itertools::Itertools;
use petgraph::{
    algo,
    dot::{Config, Dot},
    graph::{NodeIndex, UnGraph},
    Graph,
};
use std::{collections::HashMap, f32};

use crate::app::core::remove_indices;

use super::{
    core::{
        draw::{
            draw_barycenter, draw_body, draw_line_thru_points, draw_text, draw_tick_marks,
            get_scale,
        },
        physics::{
            barycenter, circ_velocity_barycenter, escape_velocity_barycenter,
            gravitational_acceleration, gravitational_potential_energy, kinetic_energy,
            symplectic_euler_calc, Acceleration, Position, Velocity,
        },
    },
    App,
};

pub struct Orbital {
    ui_state: UiState,
    pub dt: f32,
    pub num_ticks: i32,
    distance_per_px: f32,
    draw_frequency: u32, // graph a point every X seconds
    started: bool,
    stopped: bool,
    bodies: Vec<Body>,
    relationships: HashMap<usize, Vec<usize>>,
    presets: Vec<Preset>,
    analysis: Analysis,
}

impl App for Orbital {
    fn run(&mut self) {
        // let start = Instant::now();

        if !self.started || self.stopped {
            return;
        }
        for _ in 0..self.num_ticks {
            self.run_euler();
        }
        self.analyze();

        // let duration = start.elapsed();
        // println!("Run: Time elapsed = {:?}", duration);
    }

    fn draw(&self, canvas: &mut femtovg::Canvas<femtovg::renderer::WGPURenderer>) {
        // let start = Instant::now();
        let (x_distance_range, y_distance_range) = self.distance_range(canvas);
        draw_tick_marks(
            canvas,
            x_distance_range,
            y_distance_range,
            self.distance_per_px,
        );

        let ticks_per_graph_point = (self.draw_frequency as f32 / self.dt).ceil() as usize;

        for b in self.bodies.iter() {
            draw_body(canvas, b, self.distance_per_px);

            draw_line_thru_points(
                canvas,
                &b.trajectory,
                ticks_per_graph_point,
                self.distance_per_px,
                b.color,
            );

            draw_text(canvas, b.name.clone(), &b.pos, self.distance_per_px);
        }

        for bary in &self.analysis.barycenters {
            draw_barycenter(canvas, &bary, 5., self.distance_per_px);
        }
        // let duration = start.elapsed();
        // println!("Draw: Time elapsed = {:?}", duration);
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
        let mut app = Self {
            ui_state: UiState::new(),
            dt: 1.,
            num_ticks: 1000,
            distance_per_px: 150.,
            draw_frequency: 100,
            started: false,
            stopped: false,
            bodies: vec![Body::earth()],
            relationships: HashMap::new(),
            presets: Preset::defaults(),
            analysis: Analysis::default(),
        };
        app.load_preset(6);
        app
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

    fn distance_range(
        &self,
        canvas: &mut femtovg::Canvas<femtovg::renderer::WGPURenderer>,
    ) -> ((f32, f32), (f32, f32)) {
        let (width, height) = (canvas.width(), canvas.height());
        let transform = canvas.transform().0;
        let (offset_x, offset_y) = (transform[4], transform[5]);

        let scale = get_scale(canvas);
        let min_x_px = -(offset_x - self.ui_state.panel_width) / scale; // account for the side panel taking away some space
        let max_x_px = (width as f32 - offset_x) / scale;
        let max_y_px = offset_y / scale;
        let min_y_px = -(height as f32 - offset_y) / scale;

        let y_range = (
            min_y_px * self.distance_per_px,
            max_y_px * self.distance_per_px,
        );
        let x_range = (
            min_x_px * self.distance_per_px,
            max_x_px * self.distance_per_px,
        );
        (x_range, y_range)
    }

    pub fn load_preset(&mut self, preset_num: usize) {
        let preset = self.presets.get(preset_num);

        match preset {
            Some(preset) => {
                self.bodies = preset.bodies.clone();
                self.distance_per_px = preset.distance_per_px as f32;
                self.num_ticks = preset.ticks_per_press;
                self.dt = preset.dt;
                self.draw_frequency = preset.draw_frequency;
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

        let positions: Vec<(Position, Velocity, f32)> =
            self.bodies.iter().map(|b| (b.pos, b.v, b.mass)).collect();

        for body in self.bodies.iter_mut() {
            if body.lock_to_circular_velocity {
                let (locked_body_pos, _locked_body_v, locked_body_m) =
                    positions.get(body.selected_vel_lock).unwrap();

                let circ_vel =
                    circ_velocity_barycenter(body.mass, body.pos, *locked_body_m, *locked_body_pos)
                        .0;
                //.add(*_locked_body_v); // this works if the locked body velocity is not influenced by the current body (ie, hierarchical sun-earth-moon)
                // but does not work for calcualting both parts of a 2-body system
                body.v = circ_vel;
            } else if body.lock_to_escape_velocity {
                let (locked_body_pos, _locked_body_v, locked_body_m) =
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

    // returns smallest distance from a or b to a third body
    fn min_distance_to_third(&self, a: &Body, b: &Body, other: &Vec<Body>) -> f32 {
        let mut minimum = f32::INFINITY;

        for body in other.iter() {
            let cur_min = body.pos.abs_diff(a.pos).min(body.pos.abs_diff(b.pos));
            let new_min = cur_min.min(minimum);
            minimum = new_min;
        }
        minimum
    }

    // group bodies into a tree
    // each node is a body or group
    // a group can be represented by a Body (?), where the position is the barycenter, and all properties are combined together
    // each node has children - more bodies/groups

    // bodies should be grouped if M inner >> M outer, r outer >> r inner, period outer >> period inner
    fn group_bodies(&self) {
        let distance_ratio_threshold = 10.0_f32;
        let mass_ratio_threshold = 100.0_f32;

        let mut graph = UnGraph::<usize, ()>::new_undirected();
        for (i, _) in self.bodies.iter().enumerate() {
            graph.add_node(i);
        }

        for combination in self.bodies.iter().enumerate().combinations(2) {
            let (a_i, a) = combination[0];
            let (b_i, b) = combination[1];

            let other_bodies = remove_indices(self.bodies.clone(), vec![a_i, b_i]);

            println!("a: {}, b: {}", a.name, b.name);
            let other_names: Vec<&String> = other_bodies.iter().map(|b| &b.name).collect();
            println!("other: {:?}", other_names);

            let distance_between = a.pos.abs_diff(b.pos);
            let min_distance_to_third = self.min_distance_to_third(a, b, &other_bodies);

            let mass_ratio = a.mass_ratio(b);

            println!(
                "between: {}, third: {}, mass_ratio: {}",
                distance_between, min_distance_to_third, mass_ratio
            );

            if min_distance_to_third / distance_between > distance_ratio_threshold {
                if mass_ratio <= mass_ratio_threshold {
                    graph.add_edge(NodeIndex::new(a_i), NodeIndex::new(b_i), ());
                }
            }
        }

        let result = algo::tarjan_scc(&graph);
        println!("{:?}", result);

        // println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
    }

    fn test_graph(&mut self) {
        let mut graph = Graph::<i32, ()>::new();
        graph.add_node(0);
        graph.add_node(1);
        graph.add_node(2);
        graph.add_node(3);
        graph.add_node(4);
        graph.extend_with_edges(&[(1, 2), (2, 3), (3, 4), (1, 4)]);

        println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
    }

    pub fn start(&mut self) {
        // self.test_graph();
        self.group_bodies();

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

        let mut accelerations: HashMap<usize, Acceleration> = HashMap::new();

        for (affected_i, sources) in self.relationships.iter() {
            let affected = self.bodies.get(*affected_i).unwrap();

            let total_a_for_body = sources
                .iter()
                .map(|source_i| {
                    let source = self.bodies.get(*source_i).unwrap();
                    let a_from_source =
                        gravitational_acceleration(source.pos, affected.pos, source.mass);

                    a_from_source
                })
                .fold(Acceleration::default(), |acc, a| acc.add(a));

            accelerations.insert(*affected_i, total_a_for_body);
        }

        for (body_i, new_a) in accelerations.iter() {
            let affected = self.bodies.get(*body_i).unwrap();

            let cur_v = affected.v;
            let cur_r = affected.pos;
            let (next_r, next_v) = symplectic_euler_calc(cur_r, cur_v, *new_a, dt);

            self.bodies[*body_i].update(next_r, next_v, *new_a);
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
