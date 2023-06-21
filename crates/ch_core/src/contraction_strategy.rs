use crate::graph::NodeIndex;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CHStrategy<'a> {
    // Priority queue is fixed. Order of nodes is given by the slice
    FixedOrder(&'a [NodeIndex]),
    // Lazy update gets applied to the top node of the priority queue and all neighbor nodes
    #[default]
    LazyUpdateSelfAndNeighbors,
    // Lazy update only gets applied to the top node of the priority queue
    LazyUpdateSelf,
    // Lazy update gets applied to all neighbor nodes
    LazyUpdateNeighbors,
}
