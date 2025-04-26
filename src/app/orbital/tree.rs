// Modeling the group of orbital bodies as a tree,
// where each node is a body or group of bodies

use petgraph::{
    algo,
    dot::{Config, Dot},
    graph::{NodeIndex, UnGraph},
};

use crate::app::core::physics::{barycenter, Position};

use super::body::Body;

struct Node {
    pub bodies: Vec<Body>,
}

impl Node {
    fn new(body: Body) -> Self {
        Self { bodies: vec![body] }
    }
    fn pos(self) -> Position {
        // barycenter
        barycenter(self.bodies)
    }
    fn mass(self) -> f32 {
        // mass
        self.bodies.iter().fold(0., |acc, b| acc + b.mass)
    }
    fn is_leaf(self) -> bool {
        self.bodies.len() == 1
    }
}

// group bodies into a tree
// each node is a body or group
// a group can be represented by a Body (?), where the position is the barycenter, and all properties are combined together
// each node has children - more bodies/groups
// fn group_one_level()

// plan
// start at one body
// get distances to each other body in increasing order
// if there is a jump of 10x, then they can be grouped together

// bodies should be grouped if M inner >> M outer, r outer >> r inner, period outer >> period inner

// returns Vec<(index, Body, distance)> sorted by increasing distance
fn sort_by_distance(current: usize, bodies: &Vec<Body>) -> Vec<(usize, Body, f32)> {
    let current_position = bodies[current].pos;

    let mut distances = vec![];

    for (i, body) in bodies.iter().enumerate() {
        if i != current {
            let distance = current_position.abs_diff(body.pos);
            distances.push((i, body.clone(), distance));
        }
    }
    distances.sort_by(|a, b| (a.2).total_cmp(&b.2));

    distances
}

fn find_bodies_within_threshold(
    distances: &Vec<(usize, Body, f32)>,
    distance_ratio_threshold: f32,
) -> Vec<(usize, Body, f32)> {
    for (i, cur_d) in distances.iter().enumerate() {
        let next = distances.get(i + 1);

        // if there's a next distance, compare it
        if let Some(next_d) = next {
            let current_ratio = next_d.2 / cur_d.2;
            if current_ratio > distance_ratio_threshold {
                // return everything from beginning to here
                return distances[0..=i].to_vec();
            }
        }
    }

    // never found a big jump, everything is grouped together
    return distances.clone();
}

pub fn group_bodies(bodies: &Vec<Body>) {
    let distance_ratio_threshold = 10.0_f32;
    let mass_ratio_threshold = 100.0_f32;

    let mut graph = UnGraph::<usize, ()>::new_undirected();
    for (i, _) in bodies.iter().enumerate() {
        graph.add_node(i);
    }

    for (cur_i, current_body) in bodies.iter().enumerate() {
        println!("body: {}", &current_body.name);

        let bodies_sorted = sort_by_distance(cur_i, bodies);
        // find all that are "relatively close" (distance increase is less than threshold)
        let within_threshold =
            find_bodies_within_threshold(&bodies_sorted, distance_ratio_threshold);

        println!(
            "close to: {:?}",
            within_threshold
                .iter()
                .map(|(_, b, d)| format!("{}: {}", &b.name, d))
                .collect::<Vec<_>>()
        );

        for (i, body, _) in within_threshold.iter() {
            if current_body.mass_ratio(body) <= mass_ratio_threshold {
                graph.add_edge(NodeIndex::new(cur_i), NodeIndex::new(*i), ());
            }
        }
    }

    let result = algo::tarjan_scc(&graph);
    println!("{:?}", result);

    println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
}
