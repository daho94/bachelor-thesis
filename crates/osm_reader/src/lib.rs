use osmpbf::{Element, IndexedReader};
use std::{collections::HashMap, error::Error, fs::File, path::Path, str::FromStr};

mod road_types;
use road_types::RoadType;

pub struct RoadGraph {
    nodes: HashMap<i64, [f64; 2]>,
    edges: Vec<(i64, i64, f64)>,
}

impl RoadGraph {
    pub fn new() -> Self {
        RoadGraph {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, id: i64, lat: f64, lon: f64) {
        self.nodes.insert(id, [lat, lon]);
    }

    pub fn add_edge(&mut self, from: i64, to: i64, weight: f64) {
        self.edges.push((from, to, weight));
    }

    pub fn get_nodes(&self) -> &HashMap<i64, [f64; 2]> {
        &self.nodes
    }

    pub fn get_edges(&self) -> &Vec<(i64, i64, f64)> {
        &self.edges
    }

    pub fn from_pbf(pbf_path: &Path) -> Result<RoadGraph, Box<dyn Error>> {
        let mut graph = RoadGraph::new();

        let mut reader = IndexedReader::from_path(pbf_path)?;

        let road_filter = |way: &osmpbf::Way| {
            way.tags().any(|key_value| {
                key_value.0 == "highway" && key_value.1.parse::<RoadType>().is_ok()
            })
        };

        // First iteration: Only add nodes
        reader.read_ways_and_deps(road_filter, |element| match element {
            Element::Way(_) => {}
            Element::Node(node) => {
                graph.add_node(node.id(), node.lat(), node.lon());
            }
            Element::DenseNode(dense_node) => {
                graph.add_node(dense_node.id(), dense_node.lat(), dense_node.lon());
            }
            Element::Relation(_) => {}
        })?;

        // Second iteration: Add edges
        reader.read_ways_and_deps(road_filter, |element| {
            if let Element::Way(way) = element {
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

                    let [from_lat, from_lon] = graph.nodes.get(&from).unwrap();
                    let [to_lat, to_lon] = graph.nodes.get(&to).unwrap();

                    let distance = haversine_distance(*from_lat, *from_lon, *to_lat, *to_lon);

                    graph.add_edge(from, to, weight(distance, &road_type));
                }
            }
        })?;

        Ok(graph)
    }

    pub fn write_csv(&self) -> Result<(), Box<dyn Error>> {
        use std::io::Write;

        let _ = std::fs::create_dir("out");

        let mut nodes_file = File::create("out/nodes.csv")?;
        let mut edges_file = File::create("out/edges.csv")?;

        writeln!(nodes_file, "id,lat,lon")?;
        for (id, [lat, lon]) in self.nodes.iter() {
            writeln!(nodes_file, "{},{},{}", id, lat, lon)?;
        }

        writeln!(edges_file, "from,to,weight")?;
        for (from, to, weight) in self.edges.iter() {
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

// Assumes average speed in km/h for different road types and calculates the time as weight.
// v = s / t => t = s / v
fn weight(distance: f64, road_type: &RoadType) -> f64 {
    let distance_km = distance / 1000.0;
    // distance_km / road_type.velocity();
    distance
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
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn write_csv_works() {
        let filename = "test_data/minimal.osm.pbf";

        let graph = RoadGraph::from_pbf(Path::new(filename)).unwrap();
        graph.write_csv().unwrap();
    }
}
