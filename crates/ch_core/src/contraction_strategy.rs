use crate::graph::NodeIndex;

#[derive(Clone, Copy, Debug)]
pub enum ContractionStrategy<'a> {
    FixedOrder(&'a [NodeIndex]),
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

    pub fn update_jit(&self) -> bool {
        self.update_jit
    }

    pub fn update_local(&self) -> bool {
        self.update_local
    }

    pub fn update_periodic(&self) -> bool {
        self.update_periodic
    }

    pub fn set_update_jit(mut self, lazy_update_self: bool) -> Self {
        self.update_jit = lazy_update_self;
        self
    }

    pub fn set_update_local(mut self, lazy_update_neighbors: bool) -> Self {
        self.update_local = lazy_update_neighbors;
        self
    }

    pub fn set_periodic_updates(mut self, periodic_updates: bool) -> Self {
        self.update_periodic = periodic_updates;
        self
    }

    pub fn set_periodic_updates_trigger(mut self, trigger: usize) -> Self {
        self.periodic_update_data.trigger = trigger;
        self
    }

    pub fn periodic_update_triggered(&self, consecutive_lazy_updates: usize) -> bool {
        if self.update_periodic {
            consecutive_lazy_updates >= self.periodic_update_data.trigger
        } else {
            false
        }
    }
}
