use petgraph::graph::{Graph, NodeIndex};

// Returns the parent node of a given node in a directed graph.
// Returns `None` if the node has no parent (i.e., no incoming edges).
fn parent_node_idx<N, E>(graph: &Graph<N, E>, node: NodeIndex) -> Option<NodeIndex> {
    graph
        .neighbors_directed(node, petgraph::Direction::Incoming)
        .next()
}

pub fn parent_node_or_default<'a, N, E>(
    graph: &'a Graph<N, E>,
    node: NodeIndex,
    default: &'a N,
) -> &'a N {
    match parent_node_idx(graph, node) {
        Some(parent) => graph.node_weight(parent).expect("Parent node should exist"),
        None => default,
    }
}
