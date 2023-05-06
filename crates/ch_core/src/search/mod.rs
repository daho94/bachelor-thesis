use rustc_hash::FxHashMap;

use crate::graph::{DefaultIdx, IndexType, NodeIndex};

use self::shortest_path::ShortestPath;

// pub mod astar;
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
