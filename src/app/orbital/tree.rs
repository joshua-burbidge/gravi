// Modeling the group of orbital bodies as a tree,
// where each node is a body or group of bodies

use itertools::Itertools;
use petgraph::{
    algo,
    graph::{NodeIndex, UnGraph},
};

use crate::app::core::remove_indices;

use super::body::Body;

// returns smallest distance from a or b to a third body
fn min_distance_to_third(a: &Body, b: &Body, other: &Vec<Body>) -> f32 {
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
// fn group_one_level()

// bodies should be grouped if M inner >> M outer, r outer >> r inner, period outer >> period inner
pub fn group_bodies(bodies: &Vec<Body>) {
    let distance_ratio_threshold = 10.0_f32;
    let mass_ratio_threshold = 100.0_f32;

    let mut graph = UnGraph::<usize, ()>::new_undirected();
    for (i, _) in bodies.iter().enumerate() {
        graph.add_node(i);
    }

    for combination in bodies.iter().enumerate().combinations(2) {
        let (a_i, a) = combination[0];
        let (b_i, b) = combination[1];

        let other_bodies = remove_indices(bodies.clone(), vec![a_i, b_i]);

        println!("a: {}, b: {}", a.name, b.name);
        let other_names: Vec<&String> = other_bodies.iter().map(|b| &b.name).collect();
        println!("other: {:?}", other_names);

        let distance_between = a.pos.abs_diff(b.pos);
        let min_distance_to_third = min_distance_to_third(a, b, &other_bodies);

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
