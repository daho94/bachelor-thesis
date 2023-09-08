//! Search module. Contains various algorithms to calculate the shortes path between two nodes.
//!
//! This module contains the following algorithms:
//! - [`Dijkstra`]
//! - [`BidirDijkstra`]
//! - [`AStar`]
//! - [`CHSearch`]
//! # Examples
//! ```
//! use ch_core::prelude::*;
//! use ch_core::prelude::search::*;
//!
//! let mut g = generate_simple_graph();
//! let s = node_index(4);
//! let t = node_index(1);
//!
//! let mut dijk = Dijkstra::new(&g);
//! let mut bidir_dijk = BidirDijkstra::new(&g);
//!
//! assert_eq!(dijk.search(s,t), bidir_dijk.search(s,t));
//! ```
//! [`Dijkstra`]: crate::search::Dijkstra
//! [`BidirDijkstra`]: crate::search::BidirDijkstra
//! [`AStar`]: crate::search::AStar
//! [`CHSearch`]: crate::search::CHSearch
use rustc_hash::FxHashMap;

use crate::graph::NodeIndex;

pub mod astar;
pub mod bidir_dijkstra;
pub mod ch_search;
pub mod dijkstra;
pub mod shortest_path;

pub use astar::AStar;
pub use bidir_dijkstra::BidirDijkstra;
pub use ch_search::CHSearch;
pub use dijkstra::Dijkstra;
pub use shortest_path::ShortestPath;

pub fn reconstruct_path(
    target: NodeIndex,
    source: NodeIndex,
    node_data: &FxHashMap<NodeIndex, (f64, Option<NodeIndex>)>,
) -> Option<ShortestPath> {
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
fn assert_no_path(path: Option<ShortestPath>) {
    assert_eq!(None, path);
}

#[cfg(test)]
fn assert_path(
    expected_path: Vec<usize>,
    expected_weight: crate::constants::Weight,
    path: Option<ShortestPath>,
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
