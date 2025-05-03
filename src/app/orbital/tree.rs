// Modeling the group of orbital bodies as a tree,
// where each node is a body or group of bodies.

use std::{collections::HashMap, fmt::Debug};

use petgraph::{
    algo,
    dot::{Config, Dot},
    graph::{DiGraph, NodeIndex, UnGraph},
};

use crate::app::core::physics::{barycenter_abs, Position};

use super::body::Body;

#[derive(Clone)]
enum Node {
    Leaf { body: Body },
    Group { children: Vec<Node> },
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Node");

        let node_type = if self.is_leaf() { "Leaf" } else { "Group" };
        let children = match self {
            Node::Leaf { .. } => "".to_string(),
            Node::Group { children } => children.len().to_string(),
        };

        d.field("type", &node_type.to_string())
            .field("names", &self.label())
            .field("children", &children);
        d.finish()
    }
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
        self.names().join("+")
    }
    fn pos(&self) -> Position {
        match self {
            Node::Leaf { body } => body.absolute_pos,
            Node::Group { .. } => {
                let bodies = self.bodies();
                barycenter_abs(&bodies)
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
    fn is_leaf(&self) -> bool {
        match self {
            Node::Leaf { .. } => true,
            Node::Group { .. } => false,
        }
    }
}
struct Edge {
    source: usize,
    dest: usize,
}
impl Edge {
    fn new(source: usize, dest: usize) -> Self {
        Edge { source, dest }
    }
}

// group bodies into a tree
// each node is a body or group
// a group can be represented by a Body, where the position is the barycenter, and all properties are combined together
// each node has children - more bodies/groups

// bodies should be grouped if M inner >> M outer, r outer >> r inner, period outer >> period inner

// returns nodes with no incoming edges
// these are the nodes that should be considered for grouping as part of the current level
fn find_roots<N: Clone, E: Debug>(graph: &DiGraph<N, E>) -> (Vec<N>, HashMap<usize, NodeIndex>) {
    let mut map_root_idx_to_graph_idx: HashMap<usize, NodeIndex> = HashMap::new();
    let mut result = vec![];

    for ni in graph.node_indices() {
        let is_root = graph
            .edges_directed(ni, petgraph::Incoming)
            .next()
            .is_none();
        if is_root {
            result.push(
                graph
                    .node_weight(ni)
                    .expect("invalid node index - find_root")
                    .clone(),
            );
            map_root_idx_to_graph_idx.insert(result.len() - 1, ni);
        }
    }

    (result, map_root_idx_to_graph_idx)
}

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

// the graph in here is just to group bodies for this level, it's not the same graph that's used to construct the hierarchy
fn group_bodies(bodies: &Vec<Node>) -> Vec<Edge> {
    let distance_ratio_threshold = 10.0_f32;
    let mass_ratio_threshold = 100.0_f32;

    let mut edges: Vec<Edge> = vec![];

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
                edges.push(Edge::new(cur_i, *i));
            }
        }
    }

    edges
}

fn build_one_level(nodes: &Vec<Node>) -> Vec<Vec<NodeIndex>> {
    let mut graph = UnGraph::<Node, ()>::new_undirected();
    for n in nodes.iter() {
        graph.add_node(n.clone());
    }

    let edges = group_bodies(&nodes);

    for edge in edges.iter() {
        graph.add_edge(NodeIndex::new(edge.source), NodeIndex::new(edge.dest), ());
    }

    // tarjan algorithm finds all groups of connected nodes
    let groups = algo::tarjan_scc(&graph);
    println!("{:?}", groups);
    println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));

    groups
}

// Leaf node = index corresponds to original vector index
pub fn build_hierarchy(bodies: &Vec<Body>) -> (DiGraph<Body, ()>, NodeIndex) {
    let initial_nodes: Vec<Node> = bodies.iter().map(|b| Node::new(b.copy())).collect();

    let mut overall_graph = DiGraph::<Node, ()>::new();
    for (_, n) in initial_nodes.iter().enumerate() {
        overall_graph.add_node(n.clone());
    }

    let mut i = 0;
    let mut root_index: NodeIndex = NodeIndex::new(0);

    loop {
        // TODO this function should return something better - one thing instead of 2
        let (root_nodes, map_root_to_graph) = find_roots(&overall_graph);

        println!("roots: {:?}", root_nodes);
        println!("root map: {:?}", map_root_to_graph);

        i += 1;
        if i > 10 {
            println!("loop limit: probably error");
            break;
        }

        let new_groups = build_one_level(&root_nodes);

        // no more groups to make
        // TODO refactor this to share code with normal node creation
        if root_nodes.len() == new_groups.len() {
            // add the final root node
            // TODO should the root node be at 0,0 or the barycenter?
            let new_final_node = Node::Group {
                children: root_nodes.clone(),
            };
            root_index = overall_graph.add_node(new_final_node);

            for (i, _) in root_nodes.iter().enumerate() {
                let overall_idx_of_group_member = map_root_to_graph
                    .get(&i)
                    .expect("invalid mapping root node to overall graph node");
                overall_graph.add_edge(root_index, *overall_idx_of_group_member, ());
            }

            println!(
                "new graph: {:?}",
                Dot::with_config(&overall_graph, &[Config::EdgeNoLabel])
            );
            break;
        }

        // turn each group into a Group Node
        for group in new_groups.iter() {
            if group.len() > 1 {
                let nodes: Vec<Node> = group
                    .iter()
                    .map(|i| {
                        // this index actually corresponds to the nodes passed into build_one_level, not to the overall graph indices
                        root_nodes[i.index()].clone()
                    })
                    .collect();
                let new_group_node = Node::Group { children: nodes };
                let new_group_index = overall_graph.add_node(new_group_node);

                for n in group.iter() {
                    // n should be the index of root_nodes[n] in overall_graph
                    let overall_idx_of_group_member = map_root_to_graph
                        .get(&n.index())
                        .expect("invalid mapping root node to overall graph node");
                    overall_graph.add_edge(new_group_index, *overall_idx_of_group_member, ());
                }
            }
        }

        println!(
            "new graph: {:?}",
            Dot::with_config(&overall_graph, &[Config::EdgeNoLabel])
        );
    }

    println!("looped {} times", i);

    let bodies_graph = map_to_bodies(overall_graph);

    println!(
        "final body graph: {:?}",
        Dot::with_config(&bodies_graph, &[Config::EdgeNoLabel])
    );

    let localized = map_to_localized(bodies_graph);

    println!(
        "final localized graph: {:?}",
        Dot::with_config(&localized, &[Config::EdgeNoLabel])
    );

    (localized, root_index)
}

fn map_to_bodies(graph: DiGraph<Node, ()>) -> DiGraph<Body, ()> {
    let body_graph: DiGraph<Body, ()> = graph.map(
        |_nx, n| {
            let body = match n {
                Node::Leaf { body } => body.copy(),
                Node::Group { .. } => Body {
                    name: n.label() + " barycenter",
                    absolute_pos: n.pos(),
                    mass: n.mass(),
                    radius: 0.,
                    is_fixed: false,
                    is_barycenter: true,
                    color: (0, 70, 200),
                    ..Body::default()
                },
            };
            body
        },
        |_ex, _e| (),
    );

    body_graph
}

// map all bodies to localized positions, meaning
// all bodies' positions are relative to their parent
fn map_to_localized(graph: DiGraph<Body, ()>) -> DiGraph<Body, ()> {
    let localized_graph = graph.map(
        |nx, n| {
            // every node should have exactly one incoming neighbor, except the root which has zero
            let mut neighbors = graph.neighbors_directed(nx, petgraph::Direction::Incoming);
            // let parent_idx = neighbors.next().expect("no parent found");
            let localized_body = if let Some(parent_idx) = neighbors.next() {
                let parent = graph.node_weight(parent_idx).expect("invalid index");
                let localized_position = n.absolute_pos.minus(parent.absolute_pos);
                Body {
                    pos: localized_position,
                    ..n.copy()
                }
            } else {
                // if there are no neighbors, it must be the root
                Body {
                    pos: n.absolute_pos,
                    ..n.copy()
                }
            };
            localized_body
        },
        |_ex, _e| (),
    );

    localized_graph
}

// TODO map tree into a tree of Bodies - group node is a Body at the barycenter
// TODO each Body should be in a local coordinate system where 0,0 is the parent node position
