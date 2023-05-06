use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct Stats {
    pub nodes_settled: usize,
    pub duration: Option<Duration>,
    start_time: Option<Instant>,
}

impl Stats {
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

#[cfg(test)]
mod tests {
    use crate::{
        graph::{Edge, Graph, Node, NodeIndex},
        search::dijkstra::Dijkstra,
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
}
