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
        dijkstra::Dijkstra,
        graph::{Edge, GraphBuilder, Node},
    };

    #[test]
    fn stats_work() {
        //      7 -> 8 -> 9
        //      |         |
        // 0 -> 5 -> 6 -  |
        // |         |  \ |
        // 1 -> 2 -> 3 -> 4
        let nodes = (0..10).map(|i| Node::new(i, 0.0, 0.0)).collect();
        let g = GraphBuilder::new()
            .add_edge(Edge::new(0, 1, 1.0))
            .add_edge(Edge::new(1, 2, 1.0))
            .add_edge(Edge::new(2, 3, 1.0))
            .add_edge(Edge::new(3, 4, 20.0))
            .add_edge(Edge::new(0, 5, 5.0))
            .add_edge(Edge::new(5, 6, 1.0))
            .add_edge(Edge::new(6, 4, 20.0))
            .add_edge(Edge::new(6, 3, 20.0))
            .add_edge(Edge::new(5, 7, 5.0))
            .add_edge(Edge::new(7, 8, 1.0))
            .add_edge(Edge::new(8, 9, 1.0))
            .add_edge(Edge::new(9, 4, 1.0))
            .add_nodes(nodes)
            .build();

        let mut d = Dijkstra::new(&g);
        d.search(0, 4);

        assert!(d.stats.duration.is_some());

        assert_eq!(d.stats.nodes_settled, 10);
    }
}
