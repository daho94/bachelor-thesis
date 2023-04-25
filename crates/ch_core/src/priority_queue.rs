use std::collections::BinaryHeap;

use crate::constants::{NodeId, Weight};

/// Priority queue implementation using a binary heap.
/// The heap is a min heap, so the smallest element (with the lowest edge weight)
/// is always at the top.
pub struct PriorityQueue {
    heap: BinaryHeap<HeapItem>,
}

impl PriorityQueue {
    pub fn new() -> Self {
        PriorityQueue {
            heap: BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, item: HeapItem) {
        self.heap.push(item);
    }

    pub fn pop(&mut self) -> Option<HeapItem> {
        self.heap.pop()
    }
}

impl Default for PriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct HeapItem {
    pub weight: Weight,
    pub node: NodeId,
}

impl HeapItem {
    pub fn new(weight: Weight, node: NodeId) -> Self {
        HeapItem { weight, node }
    }
}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Reverse the ordering so that the smallest element is at the top of the heap.
        self.weight
            .partial_cmp(&other.weight)
            .map(|order| order.reverse())
    }
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl Eq for HeapItem {}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .weight
            .partial_cmp(&self.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}
