use std::{
    fmt::{Debug, Display},
    time::{Duration, Instant},
};

use histogram::Histogram;

use crate::graph::Graph;

#[derive(Debug, Default)]
pub struct SearchStats {
    pub nodes_settled: usize,
    pub duration: Option<Duration>,
    start_time: Option<Instant>,
}

impl SearchStats {
    pub fn init(&mut self) {
        self.nodes_settled = 0;
        self.start_timer();
    }

    fn start_timer(&mut self) {
        self.start_time = Some(Instant::now());
    }

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

pub fn degree_out_hist(g: &Graph) -> Histogram {
    degree_histogram(g, true)
}

pub fn degree_in_hist(g: &Graph) -> Histogram {
    degree_histogram(g, false)
}

pub fn average_in_degree(g: &Graph) -> f64 {
    let mut sum = 0.0;
    for node in 0..g.nodes.len() {
        sum += g.edges_in[node].len() as f64;
    }
    sum / g.nodes.len() as f64
}

pub fn average_out_degree(g: &Graph) -> f64 {
    let mut sum = 0.0;
    for node in 0..g.nodes.len() {
        sum += g.edges_out[node].len() as f64;
    }
    sum / g.nodes.len() as f64
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
