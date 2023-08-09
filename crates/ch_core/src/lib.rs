//! Crate to build contraction hierarchies.
//!
//! # Basic usage
//! ```
//! use rustc_hash::FxHashMap;
//! use std::path::Path;
//!
//! // Path to pbf file
//! let path = Path::new("path/to/pbf/file.osm.pbf");
//!
//! // Create a new graph
//! let mut g = Graph::from_pbf(&path).expect("Failed to create graph from pbf file");
//!
//! // Create a new NodeContractor instance with required parameters
//! let mut contractor = NodeContractor::new(&mut g);
//!
//! // Run the contraction algorithm
//! let overlay_graph = contractor.run();
//!
//!```
//! [`Graph`]: crate::graph::Graph
pub mod constants;
pub mod contraction_strategy;
pub mod graph;
pub mod node_contraction;
pub mod overlay_graph;
pub mod search;
pub mod statistics;
pub mod util;
pub(crate) mod witness_search;
