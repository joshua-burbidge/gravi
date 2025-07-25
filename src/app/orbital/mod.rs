pub mod body;
mod tree;
mod ui;

use body::{Body, Preset};
use log::{debug, log_enabled, Level};
use petgraph::graph::{DiGraph, NodeIndex};
use std::{collections::HashMap, f32};
use tree::build_hierarchy;

use crate::app::core::graph::parent_node_or_default;

use super::{
    core::{
        draw::{draw_body, draw_line_thru_points, draw_text, draw_tick_marks, get_scale},
        physics::{
            circ_velocity_barycenter, escape_velocity_barycenter, gravitational_acceleration,
            gravitational_potential_energy, kinetic_energy, symplectic_euler_calc, Acceleration,
            Position, Velocity,
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
    presets: Vec<Preset>,
    analysis: Analysis,
    hierarchy: DiGraph<Body, ()>,
    root: NodeIndex,
    focused: Option<NodeIndex>,
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

        if log_enabled!(Level::Debug) {
            debug!("");
            for b in self.bodies_vec().iter() {
                debug!("{}: abs: {:?}, rel: {:?}", b.name, b.absolute_pos, b.pos);
            }
        }
    }

    fn draw(&self, canvas: &mut femtovg::Canvas<femtovg::renderer::WGPURenderer>) {
        let (x_distance_range, y_distance_range) = self.distance_range(canvas);
        draw_tick_marks(
            canvas,
            x_distance_range,
            y_distance_range,
            self.distance_per_px,
        );

        let ticks_per_graph_point = (self.draw_frequency as f32 / self.dt).ceil() as usize;

        for b in self.bodies_vec().iter() {
            draw_body(canvas, b, self.distance_per_px);

            draw_line_thru_points(
                canvas,
                &b.trajectory,
                ticks_per_graph_point,
                self.distance_per_px,
                b.color,
            );

            if !b.is_barycenter {
                draw_text(
                    canvas,
                    b.name.clone(),
                    &b.absolute_pos,
                    self.distance_per_px,
                );
            }
        }
    }

    fn ui(&mut self, ctx: &egui::Context) {
        ui::ui(self, ctx);
        ui::controls_panel(self, ctx);
        if !self.started {
            // need to set velocities before so that barycenter grouping knows the bodies' velocities
            // need to set velocities after to apply the circular velocities set by the grouping
            // TODO refactor? calculate circular velocity while grouping?
            self.set_velocities();
            self.refresh_hierarchy();
            self.set_velocities();
        }
        self.analyze();
    }

    fn panel_width(&self) -> f32 {
        self.ui_state.panel_width
    }

    fn focused_pos(&self) -> Option<(f32, f32)> {
        if let Some(focused_idx) = self.focused {
            Some(
                self.hierarchy
                    .node_weight(focused_idx)
                    .expect("invalid index")
                    .absolute_pos
                    .divide(self.distance_per_px)
                    .to_tuple(),
            )
        } else {
            None
        }
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
            presets: Preset::defaults(),
            analysis: Analysis::default(),
            hierarchy: DiGraph::new(),
            root: NodeIndex::new(0),
            focused: None,
        };
        app.load_preset(0);
        app
    }

    // TODO not accurate if you change dt
    fn t(&self) -> f32 {
        let body = self.bodies_vec()[0];
        let len = body.trajectory.len();

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
        self.focused = None;

        let preset = self.presets.get(preset_num);

        match preset {
            Some(preset) => {
                // this is the one place that self.bodies should be set
                self.bodies = preset.bodies.clone();
                self.distance_per_px = preset.distance_per_px as f32;
                self.num_ticks = preset.ticks_per_press;
                self.dt = preset.dt;
                self.draw_frequency = preset.draw_frequency;
            }
            None => {}
        };
        self.create_hierarchy();
    }

    // returns the current state of the bodies - without trajectory so cloning is cheaper
    fn current_bodies(&self) -> Vec<Body> {
        self.hierarchy.node_weights().map(|b| b.copy()).collect()
    }
    fn bodies_vec(&self) -> Vec<&Body> {
        // return a vec of the bodies with same indices as graph
        self.hierarchy.node_weights().collect()
    }
    fn bodies_vec_mut(&mut self) -> Vec<&mut Body> {
        // return a vec of the bodies with same indices as graph
        self.hierarchy.node_weights_mut().collect()
    }
    fn refresh_hierarchy(&mut self) {
        let (hierarchy, root_index) = build_hierarchy(&self.original_bodies());
        self.hierarchy = hierarchy;
        self.root = root_index;
    }
    fn create_hierarchy(&mut self) {
        let (hierarchy, root_index) = build_hierarchy(&self.bodies);
        self.hierarchy = hierarchy;
        self.root = root_index;
    }
    fn bodies_list(&self) -> Vec<String> {
        self.bodies_vec()
            .iter()
            .enumerate()
            .map(|(i, b)| {
                if b.is_barycenter {
                    format!("Barycenter: {}", b.name)
                } else {
                    format!("Body {}: {}", i + 1, b.name)
                }
            })
            .collect()
    }
    fn original_bodies(&self) -> Vec<Body> {
        self.bodies_vec()
            .iter()
            .filter(|b| !b.is_barycenter)
            .cloned()
            .cloned()
            .collect()
    }

    // Set circular or orbital velocity for any body that is locked to one of those.
    // Only applies when setting initial conditions before starting.
    fn set_velocities(&mut self) {
        if self.started {
            return;
        }

        let positions: Vec<(Position, f32)> =
            self.bodies_vec().iter().map(|b| (b.pos, b.mass)).collect();

        // circ v is computing relative v
        // to convert to absolute, need to add the parent's absolute v
        let parents: Vec<_> = self
            .bodies_vec()
            .iter()
            .enumerate()
            .map(|(i, _b)| {
                let default = &Body::default();
                let parent_node =
                    parent_node_or_default(&self.hierarchy, NodeIndex::new(i), default);
                parent_node.clone()
            })
            .collect();

        // TODO better iteration over all bodies with access to parent nodes
        for (i, body) in self.bodies_vec_mut().iter_mut().enumerate() {
            if body.lock_to_circular_velocity {
                let (locked_body_pos, locked_body_m) = positions
                    .get(body.selected_vel_lock)
                    .expect("invalid index");

                let circ_vel =
                    circ_velocity_barycenter(body.mass, body.pos, *locked_body_m, *locked_body_pos)
                        .0;

                let parent_node = &parents[i];

                body.v = circ_vel;
                // loops and increases forever
                // it is circular, just the absolute v of this body and the barycenter keep changing together
                body.absolute_vel = circ_vel.add(parent_node.absolute_vel);
            } else if body.lock_to_escape_velocity {
                let (locked_body_pos, locked_body_m) = positions
                    .get(body.selected_vel_lock)
                    .expect("invalid index");

                let esc_vel = escape_velocity_barycenter(
                    body.mass,
                    body.pos,
                    *locked_body_m,
                    *locked_body_pos,
                );

                let parent_node = &parents[i];

                body.v = esc_vel;
                body.absolute_vel = esc_vel.add(parent_node.absolute_vel);
            }
        }
    }

    pub fn start(&mut self) {
        self.started = true;

        for b in self.bodies_vec_mut().iter_mut() {
            b.trajectory.push(b.copy());
        }

        self.analysis = self.analysis.initialize(self);
    }

    // Return groups of sibling bodies in BFS order.
    // Doesn't include the root node.
    // Should return Vec<&Body> or Vec<Index> or something else?
    fn sibling_groups(&self) -> (Vec<Vec<NodeIndex>>, Vec<Vec<&Body>>) {
        let mut bfs = petgraph::visit::Bfs::new(&self.hierarchy, self.root);
        let mut body_groups: Vec<Vec<&Body>> = Vec::new();
        let mut index_groups: Vec<Vec<NodeIndex>> = Vec::new();

        body_groups.push(vec![self
            .hierarchy
            .node_weight(self.root)
            .expect("invalid index")]);
        index_groups.push(vec![self.root]);

        while let Some(nx) = bfs.next(&self.hierarchy) {
            let children: Vec<NodeIndex> = self
                .hierarchy
                .neighbors_directed(nx, petgraph::Direction::Outgoing)
                .collect();
            let bodies: Vec<_> = children
                .iter()
                .map(|nx| self.hierarchy.node_weight(*nx).expect("invalid index"))
                .collect();

            if children.len() > 0 {
                body_groups.push(bodies);
                index_groups.push(children);
            }
        }

        (index_groups, body_groups)
    }

    // determine all accelerations and then update the bodies in the hierarchy
    fn hierarchical_update(&mut self) {
        let (index_groups, _) = self.sibling_groups();

        let mut updates: HashMap<NodeIndex, (Position, Velocity, Acceleration)> = HashMap::new();

        // TODO add velocity to root node when initializing, then include it in updates
        for group in index_groups.iter() {
            // calculate updates
            for &child_idx in group.iter() {
                let child = &self.hierarchy[child_idx];
                let other_children: Vec<&Body> = group
                    .iter()
                    .filter(|&&i| i != child_idx)
                    .map(|&nx| &self.hierarchy[nx])
                    .collect();

                let acceleration = self.calc_acceleration(child, other_children);

                let (next_r, next_v) =
                    symplectic_euler_calc(child.pos, child.v, acceleration, self.dt);
                updates.insert(child_idx, (next_r, next_v, acceleration));
            }

            // apply updates
            // if no parent, then it is the root node - consider 0,0 to be parent
            // (maybe add 0,0 to the tree?)
            for &node_idx in group {
                if let Some(update) = updates.get(&node_idx) {
                    let default = &Body::default(); // 0,0 position
                    let updated_parent = parent_node_or_default(&self.hierarchy, node_idx, default);

                    // parent has already been updated because it's looping in BFS order
                    let parent_abs_pos = updated_parent.absolute_pos;
                    let parent_abs_vel = updated_parent.absolute_vel;
                    let node = self
                        .hierarchy
                        .node_weight_mut(node_idx)
                        .expect("invalid index");

                    let (next_r, next_v, a) = update.clone();
                    node.update(next_r, next_v, a, parent_abs_pos, parent_abs_vel);
                } else {
                    // root node - no update
                }
            }
        }
    }

    fn calc_acceleration(&self, affected_body: &Body, sources: Vec<&Body>) -> Acceleration {
        let total_a_for_body = sources
            .iter()
            .map(|source| {
                let a_from_source =
                    gravitational_acceleration(source.pos, affected_body.pos, source.mass);

                a_from_source
            })
            .fold(Acceleration::default(), |acc, a| acc.add(a));

        total_a_for_body
    }

    // run function contains calculations necessary for the iteration process
    fn run_euler(&mut self) {
        if self.stopped {
            return;
        }

        self.hierarchical_update();

        self.check_collisions();
    }

    fn check_collisions(&mut self) -> bool {
        let bodies = self.current_bodies();

        for (i, b) in bodies.iter().enumerate() {
            for b2 in bodies[i + 1..].iter() {
                if b.is_barycenter || b2.is_barycenter {
                    continue;
                }

                let distance_between = b.absolute_pos.minus(b2.absolute_pos).mag();

                let is_collided = distance_between <= (b.radius + b2.radius);

                if is_collided {
                    self.stopped = true;
                    println!("collided: {}, {}", b.name, b2.name);
                    return true;
                }
            }
        }
        false
    }

    fn reset(&mut self) {
        let initial_bodies: Vec<Body> = self
            .original_bodies()
            .iter()
            .map(|body| match body.trajectory.get(0) {
                Some(initial_body) => initial_body.clone(),
                None => body.clone(),
            })
            .collect();

        self.bodies = initial_bodies;
        self.create_hierarchy();

        self.started = false;
        self.stopped = false;
    }

    fn set_focus(&mut self, focused: Option<NodeIndex>) {
        if let Some(focused_idx) = focused {
            self.focused = Some(focused_idx);
        } else {
            self.focused = None;
        }
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
    initial_e: f64,
    kinetic_e: f64,
    gravitational_e: f64,
    diff_percentage: f64,
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

        Analysis {
            kinetic_e: kinetic_mj,
            gravitational_e: grav_potential_mj,
            diff_percentage,
            initial_e: self.initial_e,
        }
    }

    fn current_e(&self, app: &Orbital) -> (f64, f64, f64) {
        let (_, body_groups) = app.sibling_groups();

        let (total_kinetic, total_potential) = body_groups
            .iter()
            .map(|group_bodies| {
                let (group_kinetic, group_potential) = group_bodies
                    .iter()
                    .enumerate()
                    .map(|(i, b)| {
                        let body_kinetic_mj = kinetic_energy(b.mass, b.v) as f64;

                        let body_gravitational_mj =
                            group_bodies[i + 1..].iter().fold(0., |acc, b2| {
                                let grav_potential_mj =
                                    gravitational_potential_energy(b.mass, b2.mass, b.pos, b2.pos);

                                acc + grav_potential_mj
                            });

                        (body_kinetic_mj, body_gravitational_mj)
                    })
                    .fold((0., 0.), |(acc_k, acc_g), (kinet, grav)| {
                        (acc_k + kinet, acc_g + grav)
                    });

                (group_kinetic, group_potential)
            })
            .fold((0., 0.), |(acc_k, acc_g), (kinet, grav)| {
                (acc_k + kinet, acc_g + grav)
            });

        let total = total_kinetic + total_potential;

        (total_kinetic, total_potential, total)
    }

    fn initialize(&self, app: &Orbital) -> Analysis {
        let mut initial_analysis = self.analyze(app);
        let total_e = initial_analysis.kinetic_e + initial_analysis.gravitational_e;

        initial_analysis.initial_e = total_e;

        initial_analysis
    }
}
