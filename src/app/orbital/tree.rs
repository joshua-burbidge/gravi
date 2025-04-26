// Modeling the group of orbital bodies as a tree,
// where each node is a body or group of bodies

use itertools::Itertools;
use petgraph::{
    algo,
    dot::{Config, Dot},
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

// plan
// start at one body
// get distances to each other body in increasing order
// if there is a jump of 10x, then they can be grouped together

// bodies should be grouped if M inner >> M outer, r outer >> r inner, period outer >> period inner
// this implementation won't group three bodies that are all close to each other
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

    // never found a jump big enough, so no group
    return vec![];
}

pub fn group_bodies_2(bodies: &Vec<Body>) {
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
