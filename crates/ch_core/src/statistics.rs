//! Statistics module. Used to collect various statistics.
use std::{
    fmt::{Debug, Display},
    time::{Duration, Instant},
};

use histogram::Histogram;

use crate::graph::Graph;

/// Collects statistics about the search algorithm.
#[derive(Debug, Default)]
pub struct SearchStats {
    /// Nodes visited by the search algorithm.
    pub nodes_settled: usize,
    /// Duration of the search algorithm.
    pub duration: Option<Duration>,
    start_time: Option<Instant>,
}

impl SearchStats {
    /// Resets the statistics.
    pub fn init(&mut self) {
        self.nodes_settled = 0;
        self.start_timer();
    }

    fn start_timer(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Stops the timer.
    pub fn finish(&mut self) {
        if let Some(start_time) = self.start_time {
            self.duration = Some(start_time.elapsed());
        }
    }
}

impl Display for SearchStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Stats: {} nodes settled in {:?}",
            self.nodes_settled, self.duration
        )
    }
}

fn degree_histogram(g: &Graph, outgoing: bool) -> Histogram {
    let hist = Histogram::new(0, 10, 30).unwrap();
    for node in 0..g.nodes.len() {
        if outgoing {
            let degree = g.edges_out[node].len();
            hist.increment(degree as u64, 1).unwrap();
        } else {
            let degree = g.edges_in[node].len();
            hist.increment(degree as u64, 1).unwrap();
        }
    }
    hist
}

/// Returns a histogram of the out-degree distribution of the nodes in the graph.
pub fn degree_out_hist(g: &Graph) -> Histogram {
    degree_histogram(g, true)
}

/// Returns a histogram of the in-degree distribution of the nodes in the graph.
pub fn degree_in_hist(g: &Graph) -> Histogram {
    degree_histogram(g, false)
}

/// Returns the average in-degree of the nodes in the graph.
pub fn average_in_degree(g: &Graph) -> f64 {
    let mut sum = 0.0;
    for node in 0..g.nodes.len() {
        sum += g.edges_in[node].len() as f64;
    }
    sum / g.nodes.len() as f64
}

/// Returns the average out-degree of the nodes in the graph.
pub fn average_out_degree(g: &Graph) -> f64 {
    let mut sum = 0.0;
    for node in 0..g.nodes.len() {
        sum += g.edges_out[node].len() as f64;
    }
    sum / g.nodes.len() as f64
}

/// Collects statistics about the node contraction algorithm.
#[derive(Debug, Clone, Copy)]
pub struct ConstructionStats {
    pub node_ordering_time: Duration,
    pub contraction_time: Duration,
    pub total_time: Duration,
    pub shortcuts_added: usize,
    timer: Instant,
}

impl Default for ConstructionStats {
    fn default() -> Self {
        ConstructionStats {
            node_ordering_time: Duration::new(0, 0),
            contraction_time: Duration::new(0, 0),
            total_time: Duration::new(0, 0),
            shortcuts_added: 0,
            timer: Instant::now(),
        }
    }
}

impl Display for ConstructionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "---Construction Stats---")?;
        writeln!(f, "Node Ordering      : {:?}", self.node_ordering_time)?;
        writeln!(f, "Construction       : {:?}", self.contraction_time)?;
        writeln!(f, "------------------------")?;
        writeln!(f, "Totat time         : {:?}", self.total_time)?;
        writeln!(f, "Shortcuts added [#]: {}", self.shortcuts_added)
    }
}

impl ConstructionStats {
    pub(crate) fn init(&mut self) {
        self.timer = Instant::now();
        self.shortcuts_added = 0;
        self.node_ordering_time = Duration::new(0, 0);
        self.contraction_time = Duration::new(0, 0);
        self.total_time = Duration::new(0, 0);
    }

    pub(crate) fn stop_timer_node_ordering(&mut self) {
        self.node_ordering_time = self.timer.elapsed();
        self.total_time += self.node_ordering_time;
        self.timer = Instant::now();
    }

    pub(crate) fn stop_timer_construction(&mut self) {
        self.contraction_time = self.timer.elapsed();
        self.total_time += self.contraction_time;
        self.timer = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        graph::{Edge, Graph, Node, NodeIndex},
        search::dijkstra::Dijkstra,
        statistics::{degree_in_hist, degree_out_hist},
        util::test_graphs::{graph_saarland, graph_saarland_raw},
    };

    #[test]
    fn stats_work() {
        //      7 -> 8 -> 9
        //      |         |
        // 0 -> 5 -> 6 -  |
        // |         |  \ |
        // 1 -> 2 -> 3 -> 4
        let mut g = Graph::<u32>::new();

        for i in 0..10 {
            g.add_node(Node::new(i, 0.0, 0.0));
        }

        g.add_edge(Edge::new(NodeIndex::new(0), NodeIndex::new(1), 1.0));
        g.add_edge(Edge::new(NodeIndex::new(1), NodeIndex::new(2), 1.0));
        g.add_edge(Edge::new(NodeIndex::new(2), NodeIndex::new(3), 1.0));
        g.add_edge(Edge::new(NodeIndex::new(3), NodeIndex::new(4), 20.0));
        g.add_edge(Edge::new(NodeIndex::new(0), NodeIndex::new(5), 5.0));
        g.add_edge(Edge::new(NodeIndex::new(5), NodeIndex::new(6), 1.0));
        g.add_edge(Edge::new(NodeIndex::new(6), NodeIndex::new(4), 20.0));
        g.add_edge(Edge::new(NodeIndex::new(6), NodeIndex::new(3), 20.0));
        g.add_edge(Edge::new(NodeIndex::new(5), NodeIndex::new(7), 5.0));
        g.add_edge(Edge::new(NodeIndex::new(7), NodeIndex::new(8), 1.0));
        g.add_edge(Edge::new(NodeIndex::new(8), NodeIndex::new(9), 1.0));
        g.add_edge(Edge::new(NodeIndex::new(9), NodeIndex::new(4), 1.0));

        let mut d = Dijkstra::new(&g);
        d.search(0.into(), 4.into());

        assert!(d.stats.duration.is_some());

        assert_eq!(d.stats.nodes_settled, 10);
    }

    #[test]
    fn degree_hist_out_works() {
        let g = graph_saarland();

        let hist = degree_out_hist(&g);
        dbg!(hist.buckets());
        for bucket in hist.into_iter().filter(|b| b.count() > 0) {
            println!("[{}-{}]: {}", bucket.low(), bucket.high(), bucket.count());
        }
        let g = graph_saarland_raw();

        let hist = degree_out_hist(&g);
        dbg!(hist.buckets());
        for bucket in hist.into_iter().filter(|b| b.count() > 0) {
            println!("[{}-{}]: {}", bucket.low(), bucket.high(), bucket.count());
        }
    }

    #[test]
    fn degree_hist_in_works() {
        let g = graph_saarland();

        let hist = degree_in_hist(&g);
        dbg!(hist.buckets());
        for bucket in hist.into_iter().filter(|b| b.count() > 0) {
            println!("[{}-{}]: {}", bucket.low(), bucket.high(), bucket.count());
        }
        let g = graph_saarland_raw();

        let hist = degree_in_hist(&g);
        dbg!(hist.buckets());
        for bucket in hist.into_iter().filter(|b| b.count() > 0) {
            println!("[{}-{}]: {}", bucket.low(), bucket.high(), bucket.count());
        }
    }
}
