use std::collections::BinaryHeap;

use rustc_hash::FxHashMap;

use crate::{
    constants::Weight,
    graph::{DefaultIdx, NodeIndex},
    node_contraction::NodeContractor,
};

#[derive(Debug)]
struct Candidate<Idx = DefaultIdx> {
    node_idx: NodeIndex<Idx>,
    weight: Weight,
}

impl Candidate {
    fn new(node_idx: NodeIndex, weight: Weight) -> Self {
        Self { node_idx, weight }
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.weight.partial_cmp(&self.weight)
    }
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        other.weight == self.weight
    }
}

impl Eq for Candidate {}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .weight
            .partial_cmp(&self.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

pub(crate) struct WitnessSearch<'a> {
    max_nodes_settled_limit: usize,
    node_contractor: &'a NodeContractor<'a>,
}

impl<'a> WitnessSearch<'a> {
    #[allow(dead_code)]
    pub(crate) fn new(node_contractor: &'a NodeContractor) -> Self {
        Self {
            node_contractor,
            max_nodes_settled_limit: 50,
        }
    }

    pub fn with_params(g: &'a NodeContractor, max_nodes_settled_limit: usize) -> Self {
        Self {
            node_contractor: g,
            max_nodes_settled_limit,
        }
    }

    pub(crate) fn search(
        &self,
        start: NodeIndex,
        targets: &[NodeIndex],
        avoid: NodeIndex,
        max_weight: f64,
    ) -> FxHashMap<NodeIndex, Weight> {
        let mut nodes_settled = 0;
        let mut node_data = FxHashMap::default();
        let mut targets_settled = 0;

        let mut queue = BinaryHeap::new();

        queue.push(Candidate::new(start, 0.0));

        while let Some(Candidate { weight, node_idx }) = queue.pop() {
            // Stop when all targets are settled
            if targets_settled == targets.len() {
                break;
            }

            // Stop when maximum number of nodes settled is reached
            if nodes_settled >= self.max_nodes_settled_limit {
                break;
            }

            // Stop if weight is greater than the P_max = max { <u,v,W> }
            if weight > max_weight {
                break;
            }

            for (_, edge) in self.node_contractor.neighbors_outgoing(node_idx) {
                // Skip edges where target is avoid node
                if edge.target == avoid {
                    continue;
                }

                let new_distance = weight + edge.weight;
                if new_distance < *node_data.get(&edge.target).unwrap_or(&std::f64::INFINITY) {
                    node_data.insert(edge.target, new_distance);
                    queue.push(Candidate::new(edge.target, new_distance));
                }
            }

            nodes_settled += 1;

            if targets.contains(&node_idx) {
                targets_settled += 1;
            }
        }

        node_data
    }
}
