<h2 align="center">Bachelorthesis:</h2>
<h1 align="center">Efficient routeplanning in road networks with Contraction Hierachies</h1>

<p align="center"><b>Daniel Holzner</b></p>
<p align="center"><b>17.09.2023</b></p>

---

### Summary
This thesis addresses a fundamental problem in graph theory: the computation of shortest
paths between two nodes in a graph. A practical area of application for this problem is route
planning in road networks, where the goal is to determine the quickest route between two
locations. While conventional algorithms such as Dijkstra or A⋆ are capable of handling this
task, they reach their limits when dealing with large graphs with millions of nodes and edges.
This limits their applicability, for example, in real-time navigation and
location-based services.

The Contraction Hierarchies technique, initially introduced by Geisberger et al., can overcome
these limitations. In this method, additional information is added to the graph during a
precomputation phase by adding shortcut edges. These shortcut edges are then used in the
search phase to speed up the search process. The goal of this thesis is to show, by implementing
this technique, how the computation of shortest paths in road networks can be accelerated. A
comparison of the results of this technique with traditional methods such as Dijkstra or A⋆ will
also be part of the research.

The implementation was realised in the Rust programming language and is available as a Rust
library. The runtime analysis shows a significant improvement in execution speed compared
to Dijkstra and A⋆. All measurements were performed on real road data from the OpenData
project OpenStreetMap. In large road networks, comparable to the size of Germany (10 million
nodes and 22 million edges), path calculations over long distances can be performed in less
than one millisecond by using Contraction Hierarchies, which results in an improvement factor
of more than 1000 compared to the standard methods.

---
### Basic usage
```rust
use std::path::Path;
use ch_core::prelude::*;

// Path to pbf file
let path = Path::new("path/to/pbf/file.osm.pbf");

// Create a new graph
let mut g = Graph::from_pbf(&path).expect("Failed to create graph from pbf file");

// Create a new NodeContractor instance with required parameters
let mut contractor = NodeContractor::new(&mut g);

// Run the contraction algorithm
let overlay_graph = contractor.run();

// Search
let mut ch = search::CHSearch::new(&overlay_graph);
let s = node_index(3);
let t = node_index(20);

let shortest_path = ch.search(s, t).expect("Failed to find path");
println!("Costs: {}", shortest_path.weight);
```

---
### Performance
<p align="center">
<img src="assets/boxplot_rank_log.png" alt="drawing" width="80%"/>
</p>

--- 
### Search space

<p float="left" align="center">
  <img src="assets/search_spaces.png" width="80%"/>
</p>

---
### Visualization
https://github.com/daho94/bachelor-thesis/assets/20201570/ae85c29c-2416-48cc-94a0-3bae61dae7be

