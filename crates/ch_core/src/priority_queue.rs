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

#[derive(Copy, Clone, Debug)]
pub struct HeapItem {
    pub distance: Weight,
    pub node: NodeId,
}

impl HeapItem {
    pub fn new(distance: Weight, node: NodeId) -> Self {
        HeapItem { distance, node }
    }
}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Reverse the ordering so that the smallest element is at the top of the heap.
        self.distance
            .partial_cmp(&other.distance)
            .map(|order| order.reverse())
    }
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for HeapItem {}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .distance
            .partial_cmp(&self.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}
