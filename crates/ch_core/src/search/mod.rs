use rustc_hash::FxHashMap;

use crate::graph::{IndexType, NodeIndex};

use self::shortest_path::ShortestPath;

pub mod astar;
pub mod dijkstra;
pub mod shortest_path;

pub fn reconstruct_path<Idx: IndexType>(
    target: NodeIndex<Idx>,
    source: NodeIndex<Idx>,
    node_data: &FxHashMap<NodeIndex<Idx>, (f64, Option<NodeIndex<Idx>>)>,
) -> Option<ShortestPath<Idx>> {
    let mut path = vec![target];
    let weight = node_data.get(&target)?.0;

    let mut previous_node = node_data.get(&target)?.1?;

    while let Some(prev_node) = node_data.get(&previous_node)?.1 {
        path.push(previous_node);
        previous_node = prev_node;
    }
    path.push(source);
    path.reverse();
    Some(ShortestPath::new(path, weight))
}

#[cfg(test)]
fn assert_no_path(path: Option<ShortestPath<crate::graph::DefaultIdx>>) {
    assert_eq!(None, path);
}

#[cfg(test)]
fn assert_path(
    expected_path: Vec<usize>,
    expected_weight: crate::constants::Weight,
    path: Option<ShortestPath<crate::graph::DefaultIdx>>,
) {
    assert_eq!(
        Some(ShortestPath::new(
            expected_path
                .iter()
                .map(|i| crate::graph::node_index(*i))
                .collect(),
            expected_weight
        )),
        path
    );
}
