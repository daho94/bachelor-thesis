use rustc_hash::FxHashMap;

use self::shortest_path::ShortestPath;

pub mod astar;
pub mod dijkstra;
pub mod shortest_path;

pub fn reconstruct_path(
    dst: usize,
    src: usize,
    node_data: &FxHashMap<usize, (f64, Option<usize>)>,
) -> Option<ShortestPath> {
    let mut path = vec![dst];
    let weight = node_data.get(&dst)?.0;

    let mut previous_node = node_data.get(&dst)?.1?;

    while let Some(prev_node) = node_data.get(&previous_node)?.1 {
        path.push(previous_node);
        previous_node = prev_node;
    }
    path.push(src);
    path.reverse();
    Some(ShortestPath::new(path, weight))
}
