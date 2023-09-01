use crate::graph::NodeIndex;

#[derive(Clone, Copy, Debug)]
pub enum CHStrategy<'a> {
    FixedOrder(&'a [NodeIndex]),
    LazyUpdate(UpdateStrategy),
}

impl Default for CHStrategy<'_> {
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
    update_top: bool,
    update_neighbors: bool,
    update_periodic: bool,

    periodic_update_data: PeriodicUpdateData,
}

impl Default for UpdateStrategy {
    fn default() -> Self {
        Self {
            update_top: true,
            update_neighbors: true,
            update_periodic: false,
            periodic_update_data: PeriodicUpdateData::default(),
        }
    }
}

impl UpdateStrategy {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_top(&self) -> bool {
        self.update_top
    }

    pub fn update_neighbors(&self) -> bool {
        self.update_neighbors
    }

    pub fn update_periodic(&self) -> bool {
        self.update_periodic
    }

    pub fn set_update_top(mut self, lazy_update_self: bool) -> Self {
        self.update_top = lazy_update_self;
        self
    }

    pub fn set_update_neighbors(mut self, lazy_update_neighbors: bool) -> Self {
        self.update_neighbors = lazy_update_neighbors;
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
