//! Module to choose a strategy for the node contraction process.
//!
//! # Examples
//! ```
//! use ch_core::prelude::*;
//!
//! // Use a fixed order for contraction
//! let contraction_strategy = ContractionStrategy::FixedOrder(&[node_index(0), node_index(1)]);
//!
//! // Use a lazy update strategy with periodic updates
//! let update_strategy = UpdateStrategy::new().set_periodic_updates(true);
//!
//! let contraction_strategy = ContractionStrategy::LazyUpdate(update_strategy);
//! ```
use crate::graph::NodeIndex;

/// Strategy which is used while contracting nodes.
#[derive(Clone, Copy, Debug)]
pub enum ContractionStrategy<'a> {
    /// Nodes are contracted in the `exact` order of the given slice.
    FixedOrder(&'a [NodeIndex]),
    /// The order gets updated according to the chosen [UpdateStrategy] while the contraction process is running.
    LazyUpdate(UpdateStrategy),
}

impl Default for ContractionStrategy<'_> {
    fn default() -> Self {
        Self::LazyUpdate(UpdateStrategy::default())
    }
}

#[derive(Clone, Copy, Debug)]
struct PeriodicUpdateData {
    /// If `trigger` consecutive lazy updates happen, trigger a full update
    trigger: usize,
    #[allow(dead_code)]
    // If frequence is 1, trigger at 50%, if 2 at 25% and 75% and so on
    // For now the update happens at 50% of nodes contracted.
    frequency: usize,
}

impl Default for PeriodicUpdateData {
    fn default() -> Self {
        Self {
            trigger: 200,
            frequency: 1,
        }
    }
}

/// Strategy which is used to update the contraction order.
#[derive(Clone, Copy, Debug)]
pub struct UpdateStrategy {
    update_jit: bool,
    update_local: bool,
    update_periodic: bool,

    periodic_update_data: PeriodicUpdateData,
}

impl Default for UpdateStrategy {
    fn default() -> Self {
        Self {
            update_jit: true,
            update_local: true,
            update_periodic: false,
            periodic_update_data: PeriodicUpdateData::default(),
        }
    }
}

impl UpdateStrategy {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the `Just In Time`-Strategy is enabled
    pub fn update_jit(&self) -> bool {
        self.update_jit
    }

    /// Returns true if the `Local Update`-Strategy is enabled
    pub fn update_local(&self) -> bool {
        self.update_local
    }

    /// Returns true if the `Periodic Update`-Strategy is enabled
    pub fn update_periodic(&self) -> bool {
        self.update_periodic
    }

    /// Enable or disable the `Just In Time`-Strategy
    pub fn set_update_jit(mut self, lazy_update_self: bool) -> Self {
        self.update_jit = lazy_update_self;
        self
    }

    /// Enable or disable the `Local Update`-Strategy
    pub fn set_update_local(mut self, lazy_update_neighbors: bool) -> Self {
        self.update_local = lazy_update_neighbors;
        self
    }

    /// Enable or disable the `Periodic Update`-Strategy
    pub fn set_periodic_updates(mut self, periodic_updates: bool) -> Self {
        self.update_periodic = periodic_updates;
        self
    }

    #[allow(dead_code)]
    fn set_periodic_updates_trigger(mut self, trigger: usize) -> Self {
        self.periodic_update_data.trigger = trigger;
        self
    }

    #[allow(dead_code)]
    fn periodic_update_triggered(&self, consecutive_lazy_updates: usize) -> bool {
        if self.update_periodic {
            consecutive_lazy_updates >= self.periodic_update_data.trigger
        } else {
            false
        }
    }
}
