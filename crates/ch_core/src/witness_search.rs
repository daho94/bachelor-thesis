use std::collections::BinaryHeap;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    constants::Weight,
    graph::{DefaultIdx, EdgeIndex, Graph, IndexType, NodeIndex},
};

#[derive(Debug)]
struct Candidate<Idx = DefaultIdx> {
    node_idx: NodeIndex<Idx>,
    weight: Weight,
}

impl<Idx: IndexType> Candidate<Idx> {
    fn new(node_idx: NodeIndex<Idx>, weight: Weight) -> Self {
        Self { node_idx, weight }
    }
}

impl<Idx: IndexType> PartialOrd for Candidate<Idx> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.weight.partial_cmp(&self.weight)
    }
}

impl<Idx: IndexType> PartialEq for Candidate<Idx> {
    fn eq(&self, other: &Self) -> bool {
        other.weight == self.weight
    }
}

impl<Idx: IndexType> Eq for Candidate<Idx> {}

impl<Idx: IndexType> Ord for Candidate<Idx> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .weight
            .partial_cmp(&self.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

pub(crate) struct WitnessSearch<'a> {
    max_nodes_settled: usize,
    g: &'a Graph,
}

impl<'a> WitnessSearch<'a> {
    pub(crate) fn new(g: &'a Graph) -> Self {
        Self {
            g,
            max_nodes_settled: 50,
        }
    }

    pub(crate) fn search(
        &self,
        start: NodeIndex,
        targets: &[NodeIndex],
        avoid: NodeIndex,
        max_weight: f64,
        ignore: &FxHashSet<EdgeIndex>,
    ) -> FxHashMap<NodeIndex, Weight> {
        let mut nodes_settled = 0;
        let mut node_data = FxHashMap::default();
        let mut targets_settled = 0;

        let mut queue = BinaryHeap::new();
        let mut settled = FxHashSet::default();

        queue.push(Candidate::new(start, 0.0));

        while let Some(Candidate { weight, node_idx }) = queue.pop() {
            if nodes_settled >= self.max_nodes_settled {
                dbg!("Max nodes settled limit reached");
                break;
            }

            if weight > max_weight {
                break;
            }

            for (_, edge) in self
                .g
                .neighbors_outgoing(node_idx)
                .filter(|(i, _)| !ignore.contains(i))
            {
                // Skip edges where target is avoid node
                if edge.target == avoid
                // || settled.contains(&edge.target)
                {
                    continue;
                }

                let new_distance = weight + edge.weight;
                if new_distance < *node_data.get(&edge.target).unwrap_or(&std::f64::INFINITY) {
                    node_data.insert(edge.target, new_distance);
                    queue.push(Candidate::new(edge.target, new_distance));
                }
            }

            nodes_settled += 1;
            settled.insert(node_idx);
            if targets.contains(&node_idx) {
                targets_settled += 1;
            }

            // If all targets are settled
            if targets_settled == targets.len() {
                break;
            }
        }

        node_data
    }
}
