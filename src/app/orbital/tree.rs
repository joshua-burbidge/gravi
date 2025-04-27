// Modeling the group of orbital bodies as a tree,
// where each node is a body or group of bodies

use petgraph::{
    algo,
    dot::{Config, Dot},
    graph::{NodeIndex, UnGraph},
};

use crate::app::core::physics::{barycenter, Position};

use super::body::Body;

#[derive(Clone, Debug)]
enum Node {
    Leaf { body: Body },
    Group { children: Vec<Node> },
}

impl Node {
    fn new(body: Body) -> Self {
        Self::Leaf { body }
    }
    fn names(&self) -> Vec<String> {
        let names = match self {
            Node::Leaf { body } => vec![body.name.clone()],
            Node::Group { children, .. } => children.iter().flat_map(|c| c.names()).collect(),
        };
        names
    }
    fn label(&self) -> String {
        self.names().join(", ")
    }
    fn pos(&self) -> Position {
        match self {
            Node::Leaf { body } => body.pos,
            Node::Group { .. } => {
                let bodies = self.bodies();
                barycenter(bodies)
            }
        }
    }
    fn bodies(&self) -> Vec<Body> {
        match self {
            Node::Leaf { body } => vec![body.copy()],
            Node::Group { children } => children.iter().flat_map(|n| n.bodies()).collect(),
        }
    }
    fn mass(&self) -> f32 {
        match self {
            Node::Leaf { body } => body.mass,
            Node::Group { children, .. } => children.iter().map(|c| c.mass()).sum(),
        }
    }
    fn mass_ratio(&self, other: &Node) -> f32 {
        let self_mass = self.mass();
        let other_mass = other.mass();

        (self_mass / other_mass).max(other_mass / self_mass)
    }
    fn _is_leaf(self) -> bool {
        match self {
            Node::Leaf { .. } => true,
            Node::Group { .. } => false,
        }
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
fn sort_by_distance(current: usize, bodies: &Vec<Node>) -> Vec<(usize, Node, f32)> {
    let current_position = bodies[current].pos();

    let mut distances = vec![];

    for (i, body) in bodies.iter().enumerate() {
        if i != current {
            let distance = current_position.abs_diff(body.pos());
            distances.push((i, body.clone(), distance));
        }
    }
    distances.sort_by(|a, b| (a.2).total_cmp(&b.2));

    distances
}

fn find_bodies_within_threshold(
    distances: &Vec<(usize, Node, f32)>,
    distance_ratio_threshold: f32,
) -> Vec<(usize, Node, f32)> {
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
    // return distances.clone();
    // that approach seems weird - i wouldn't want to group three different planets around the sun right?
    return vec![];
}

// the graph in here is just to group bodies, it's not the same graph that's used to construct the hierarchy
fn group_bodies(bodies: &Vec<Node>) -> Vec<(usize, usize)> {
    let distance_ratio_threshold = 10.0_f32;
    let mass_ratio_threshold = 100.0_f32;

    let mut edges: Vec<(usize, usize)> = vec![];

    for (cur_i, current_node) in bodies.iter().enumerate() {
        println!("current node: {:?}", current_node.names());

        let bodies_sorted = sort_by_distance(cur_i, bodies);
        println!(
            "sorted nodes: {:?}",
            bodies_sorted
                .iter()
                .map(|(i, n, d)| format!("{} - {}, d: {}", i, n.label(), d))
                .collect::<Vec<String>>()
        );

        // find all that are "relatively close" (distance increase is less than threshold)
        let within_threshold =
            find_bodies_within_threshold(&bodies_sorted, distance_ratio_threshold);

        println!(
            "close to: {:?}",
            within_threshold
                .iter()
                .map(|(_, b, d)| format!("{:?}: {}", &b.names(), d))
                .collect::<Vec<_>>()
        );

        for (i, body, _) in within_threshold.iter() {
            if current_node.mass_ratio(body) <= mass_ratio_threshold {
                edges.push((cur_i, *i));
            }
        }
    }

    edges
}

fn build_one_level(nodes: &Vec<Node>) -> Vec<Vec<NodeIndex>> {
    let mut graph = UnGraph::<String, ()>::new_undirected();
    for (_, n) in nodes.iter().enumerate() {
        graph.add_node(n.label());
    }

    let edges = group_bodies(&nodes);

    for (start, end) in edges.iter() {
        graph.add_edge(NodeIndex::new(*start), NodeIndex::new(*end), ());
    }

    // tarjan algorithm finds all groups of connected nodes
    let groups = algo::tarjan_scc(&graph);
    println!("{:?}", groups);
    println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));

    groups
}

pub fn build_hierarchy(bodies: &Vec<Body>) {
    let initial_nodes: Vec<Node> = bodies.iter().map(|b| Node::new(b.copy())).collect();

    let mut current_nodes = initial_nodes;
    let mut i = 0;

    loop {
        i += 1;
        if i > 10 {
            println!("loop limit: probably error");
            break;
        }

        let new_groups = build_one_level(&current_nodes);

        if current_nodes.len() == new_groups.len() {
            break;
        }

        // turn each group into a Group Node
        let mut new_nodes: Vec<Node> = vec![];
        for group in new_groups.iter() {
            let group_node = if group.len() > 1 {
                let group_nodes: Vec<Node> = group
                    .iter()
                    .map(|nx| current_nodes[nx.index()].clone())
                    .collect();

                let new_node: Node = Node::Group {
                    children: group_nodes,
                };
                new_node
            } else if group.len() == 1 {
                let nx = group[0];
                current_nodes[nx.index()].clone()
            } else {
                panic!("group of length zero")
            };
            new_nodes.push(group_node);
        }

        current_nodes = new_nodes;
    }

    println!("looped {} times", i);
}
