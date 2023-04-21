use osmpbf::{Element, IndexedReader};
use std::{collections::HashMap, fs::File, path::Path, str::FromStr};

mod road_types;
use road_types::RoadType;

pub struct RoadGraph {
    nodes: HashMap<i64, [f64; 2]>,
    arcs: Vec<(i64, i64, f64)>,
}

impl RoadGraph {
    pub fn new() -> Self {
        RoadGraph {
            nodes: HashMap::new(),
            arcs: Vec::new(),
        }
    }

    pub fn add_node(&mut self, id: i64, lat: f64, lon: f64) {
        self.nodes.insert(id, [lat, lon]);
    }

    pub fn add_arc(&mut self, from: i64, to: i64, weight: f64) {
        self.arcs.push((from, to, weight));
    }

    pub fn get_nodes(&self) -> &HashMap<i64, [f64; 2]> {
        &self.nodes
    }

    pub fn get_arcs(&self) -> &Vec<(i64, i64, f64)> {
        &self.arcs
    }

    pub fn from_pbf(pbf_path: &Path) -> anyhow::Result<RoadGraph> {
        let mut graph = RoadGraph::new();

        let mut reader = IndexedReader::from_path(pbf_path)?;

        let road_filter = |way: &osmpbf::Way| {
            way.tags()
                .any(|(key, value)| key == "highway" && value.parse::<RoadType>().is_ok())
        };

        let mut edges = Vec::new();

        // First iteration: Only add nodes
        reader.read_ways_and_deps(road_filter, |element| match element {
            Element::Way(way) => {
                let node_ids = way.refs().collect::<Vec<_>>();
                let tags = way.tags().collect::<Vec<_>>();

                // Find tag "highway" and extract value
                let road_type = tags
                    .iter()
                    .find(|key_value| key_value.0 == "highway")
                    .unwrap()
                    .1;
                let road_type = RoadType::from_str(road_type).unwrap();

                for i in 0..node_ids.len() - 1 {
                    let from = node_ids[i];
                    let to = node_ids[i + 1];

                    // For now all arcs are bidirectional
                    edges.push((from, to, road_type));
                    edges.push((to, from, road_type));
                }
            }
            Element::Node(node) => {
                graph.add_node(node.id(), node.lat(), node.lon());
            }
            Element::DenseNode(dense_node) => {
                graph.add_node(dense_node.id(), dense_node.lat(), dense_node.lon());
            }
            Element::Relation(_) => {}
        })?;

        // Calculate weights and add arcs to graph
        graph.arcs = Vec::with_capacity(edges.len());
        for (from, to, road_type) in edges {
            let [from_lat, from_lon] = graph.nodes.get(&from).unwrap();
            let [to_lat, to_lon] = graph.nodes.get(&to).unwrap();

            let distance = haversine_distance(*from_lat, *from_lon, *to_lat, *to_lon);

            graph.add_arc(from, to, weight(distance, &road_type));
        }

        Ok(graph)
    }

    pub fn write_csv(&self) -> anyhow::Result<()> {
        use std::io::Write;

        let mut nodes_file = File::create("nodes.csv")?;
        let mut edges_file = File::create("edges.csv")?;

        writeln!(nodes_file, "id,lat,lon")?;
        for (id, [lat, lon]) in self.nodes.iter() {
            writeln!(nodes_file, "{},{},{}", id, lat, lon)?;
        }

        writeln!(edges_file, "from,to,weight")?;
        for (from, to, weight) in self.arcs.iter() {
            writeln!(edges_file, "{},{},{}", from, to, weight)?;
        }

        Ok(())
    }
}

impl Default for RoadGraph {
    fn default() -> Self {
        Self::new()
    }
}

// Assumes average speed in km/h for different road types and calculates the time [sec] as weight.
// v = s / t => t = s / v
fn weight(distance: f64, road_type: &RoadType) -> f64 {
    // velocity km/h in m/s
    let velocity = road_type.velocity() / 3.6;
    distance / velocity
}

// Calculates the great-circle distance between two points in metres
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6_378_100.0; // FIXME: Find good radius for germany
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + (d_lon / 2.0).sin().powi(2) * lat1.to_radians().cos() * lat2.to_radians().cos();
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    r * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_from_pbf_works() {
        let filename = "test_data/minimal.osm.pbf";

        let graph = RoadGraph::from_pbf(Path::new(filename)).unwrap();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.arcs.len(), 2);
    }

    #[test]
    fn write_csv_works() {
        let filename = "test_data/minimal.osm.pbf";

        let graph = RoadGraph::from_pbf(Path::new(filename)).unwrap();
        graph.write_csv().unwrap();
    }
}
