//! Parameters for the contraction algorithm

/// Parameters for the contraction algorithm
#[derive(Debug, Clone, Copy)]
pub struct ContractionParams {
    pub(crate) priority_params: PriorityParams,
    // Limit for lazy updates
    pub(crate) witness_search_limit: usize,
    // Limit for initial node ordering
    pub(crate) witness_search_initial_limit: usize,
}

impl ContractionParams {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn priority_params(mut self, params: PriorityParams) -> Self {
        self.priority_params = params;
        self
    }

    pub fn witness_search_limit(mut self, limit: usize) -> Self {
        self.witness_search_limit = limit;
        self
    }

    pub fn witness_search_initial_limit(mut self, limit: usize) -> Self {
        self.witness_search_initial_limit = limit;
        self
    }
}

impl Default for ContractionParams {
    fn default() -> Self {
        ContractionParams {
            priority_params: Default::default(),
            witness_search_limit: 50,
            witness_search_initial_limit: 500,
        }
    }
}

/// Coefficients for the priority function
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriorityParams {
    pub edge_difference_coeff: i32,
    pub contracted_neighbors_coeff: i32,
    pub search_space_coeff: i32,
    pub original_edges_coeff: i32,
}

impl PriorityParams {
    pub fn new(
        edge_difference_coeff: i32,
        contracted_neighbors_coeff: i32,
        search_space_coeff: i32,
        original_edges_coeff: i32,
    ) -> Self {
        PriorityParams {
            edge_difference_coeff,
            contracted_neighbors_coeff,
            search_space_coeff,
            original_edges_coeff,
        }
    }

    pub fn edge_difference_coeff(mut self, coeff: i32) -> Self {
        self.edge_difference_coeff = coeff;
        self
    }

    pub fn contracted_neighbors_coeff(mut self, coeff: i32) -> Self {
        self.contracted_neighbors_coeff = coeff;
        self
    }

    pub fn search_space_coeff(mut self, coeff: i32) -> Self {
        self.search_space_coeff = coeff;
        self
    }

    pub fn original_edges_coeff(mut self, coeff: i32) -> Self {
        self.original_edges_coeff = coeff;
        self
    }
}

// From Diploma thesis Contraction Hierarchies - Geisberger
// edge_difference_coeff: 190,
// contracted_neighbors_coeff: 120,
// search_space_coeff: 1,
// original_edges_coeff: 70,
//
// From Raster Search - Vaterstetten:
// edge_difference_coeff: 101,
// contracted_neighbors_coeff: 101,
// search_space_coeff: 6,
// original_edges_coeff: 70,
//
// From Raster Search - Saarland:
// Best aggressive params: PriorityParams {
//     edge_difference_coeff: 501,
//     contracted_neighbors_coeff: 401,
//     search_space_coeff: 7,
//     original_edges_coeff: 201,
// } with averagy query time: 75 μs
impl Default for PriorityParams {
    fn default() -> Self {
        PriorityParams {
            edge_difference_coeff: 501,
            contracted_neighbors_coeff: 401,
            search_space_coeff: 7,
            original_edges_coeff: 201,
        }
    }
}
