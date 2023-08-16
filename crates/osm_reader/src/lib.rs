use log::info;
use osmpbf::{Element, IndexedReader};
use rustc_hash::FxHashMap;
use std::{collections::HashMap, fs::File, io::BufWriter, path::Path, str::FromStr};

mod road_types;
use road_types::RoadType;

pub struct Arc {
    pub source: i64,
    pub target: i64,
    pub weight: f64,
}

impl Arc {
    fn new(source: i64, target: i64, weight: f64) -> Self {
        Self {
            source,
            target,
            weight,
        }
    }
}

pub struct RoadGraph {
    nodes: FxHashMap<i64, [f64; 2]>,
    arcs: Vec<Arc>,
}

impl RoadGraph {
    pub fn new() -> Self {
        RoadGraph {
            nodes: FxHashMap::default(),
            arcs: Vec::new(),
        }
    }

    pub fn add_node(&mut self, id: i64, lat: f64, lon: f64) {
        self.nodes.insert(id, [lat, lon]);
    }

    pub fn add_arc(&mut self, from: i64, to: i64, weight: f64) {
        self.arcs.push(Arc::new(from, to, weight));
    }

    pub fn get_nodes(&self) -> &FxHashMap<i64, [f64; 2]> {
        &self.nodes
    }

    pub fn get_arcs(&self) -> &Vec<Arc> {
        &self.arcs
    }

    pub fn from_pbf_without_geometry(pbf_path: &Path) -> anyhow::Result<RoadGraph> {
        let mut graph = RoadGraph::new();

        let mut reader = IndexedReader::from_path(pbf_path)?;

        let road_filter = |way: &osmpbf::Way| {
            way.tags()
                .any(|(key, value)| key == "highway" && value.parse::<RoadType>().is_ok())
        };

        // let mut edges = Vec::new();
        let mut refs_count = HashMap::new();
        let mut ways = Vec::new();

        let mut nodes: FxHashMap<i64, [f64; 2]> = Default::default();

        let now = std::time::Instant::now();
        info!("BEGIN parsing {}", pbf_path.display());
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

                let is_oneway = {
                    if let Some((_, value)) = tags.iter().find(|(key, _)| *key == "oneway") {
                        match *value {
                            // Tag always has prio if explicitly set
                            "yes" => true,
                            "no" => false,
                            // If no tag is found check the road type
                            _ => road_type.is_oneway(),
                        }
                    } else {
                        false
                    }
                };

                (0..node_ids.len()).for_each(|i| {
                    let from = node_ids[i];

                    if i == 0 || i == node_ids.len() - 1 {
                        // Start and End of a road should always be included
                        *refs_count.entry(from).or_insert(0) += 2;
                    } else {
                        *refs_count.entry(from).or_insert(0) += 1;
                    }
                });

                ways.push((node_ids, road_type, is_oneway));
            }
            Element::Node(node) => {
                nodes.insert(node.id(), [node.lat(), node.lon()]);
            }
            Element::DenseNode(dense_node) => {
                nodes.insert(dense_node.id(), [dense_node.lat(), dense_node.lon()]);
            }
            Element::Relation(_) => {}
        })?;
        info!("FINISHED parsing. Took {:?}", now.elapsed());

        let now = std::time::Instant::now();
        info!("BEGIN simplyfing graph");

        graph.arcs = Vec::with_capacity(ways.len() * 2);
        // Split ways, but only keep nodes that are referenced more than once
        for (node_ids, road_type, is_oneway) in ways {
            let mut nodes_to_keep = vec![];
            (0..node_ids.len()).for_each(|i| {
                let node_id = node_ids[i];
                if refs_count.get(&node_id).unwrap() > &1 {
                    nodes_to_keep.push(i);
                }
            });

            // Add nodes to graph
            for i in nodes_to_keep.iter() {
                let node_id = node_ids[*i];
                let [lat, lon] = nodes.get(&node_id).unwrap();
                graph.add_node(node_id, *lat, *lon);
            }

            for i in 0..nodes_to_keep.len() - 1 {
                let from = nodes_to_keep[i];
                let to = nodes_to_keep[i + 1];

                let mut total_weight = 0.0;
                for j in from..to {
                    let [from_lat, from_lon] = nodes.get(&node_ids[j]).unwrap();
                    let [to_lat, to_lon] = nodes.get(&node_ids[j + 1]).unwrap();
                    let distance = haversine_distance(*from_lat, *from_lon, *to_lat, *to_lon);

                    total_weight += weight(distance, &road_type);
                }

                graph.add_arc(node_ids[from], node_ids[to], total_weight);
                // If bidirectional add reverse edge
                if !is_oneway {
                    graph.add_arc(node_ids[to], node_ids[from], total_weight);
                }
            }
        }
        info!("FINISHED simplyfing graph. Took {:?}", now.elapsed());

        Ok(graph)
    }

    pub fn from_pbf(pbf_path: &Path) -> anyhow::Result<RoadGraph> {
        let mut graph = RoadGraph::new();

        let mut reader = IndexedReader::from_path(pbf_path)?;

        let road_filter = |way: &osmpbf::Way| {
            way.tags()
                .any(|(key, value)| key == "highway" && value.parse::<RoadType>().is_ok())
        };

        let mut edges = Vec::new();

        let now = std::time::Instant::now();
        info!("BEGIN parsing {}", pbf_path.display());
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

                let is_oneway = {
                    if let Some((_, value)) = tags.iter().find(|(key, _)| *key == "oneway") {
                        match *value {
                            // Tag always has prio if explicitly set
                            "yes" => true,
                            "no" => false,
                            // If no tag is found check the road type
                            _ => road_type.is_oneway(),
                        }
                    } else {
                        false
                    }
                };

                for i in 0..node_ids.len() - 1 {
                    let from = node_ids[i];
                    let to = node_ids[i + 1];

                    edges.push((from, to, road_type));

                    // If bidirectional add reverse edge
                    if !is_oneway {
                        edges.push((to, from, road_type));
                    }
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
        graph.arcs = Vec::new();
        for (from, to, road_type) in edges {
            let [from_lat, from_lon] = graph.nodes.get(&from).unwrap();
            let [to_lat, to_lon] = graph.nodes.get(&to).unwrap();

            let distance = haversine_distance(*from_lat, *from_lon, *to_lat, *to_lon);

            graph.add_arc(from, to, weight(distance, &road_type));
        }

        info!("FINISHED parsing. Took {:?}", now.elapsed());
        Ok(graph)
    }

    pub fn write_csv(&self) -> anyhow::Result<()> {
        use std::io::Write;

        let nodes_file = File::create("nodes.csv")?;
        let edges_file = File::create("edges.csv")?;

        let mut nodes_writer = BufWriter::new(nodes_file);
        let _ = nodes_writer.write("id,lat,lon\n".as_bytes())?;
        for (id, [lat, lon]) in self.nodes.iter() {
            let _ = nodes_writer.write(format!("{},{},{}\n", id, lat, lon).as_bytes())?;
        }
        nodes_writer.flush()?;

        let mut edges_writer = BufWriter::new(edges_file);
        let _ = edges_writer.write("source,target,weight\n".as_bytes())?;
        for Arc {
            weight,
            source,
            target,
        } in self.arcs.iter()
        {
            let _ = edges_writer.write(format!("{},{},{}\n", source, target, weight).as_bytes())?;
        }
        edges_writer.flush()?;

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

    #[test]
    fn graph_from_pbf_without_geometry_works() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data/node_refs.osm.pbf");

        let graph = RoadGraph::from_pbf_without_geometry(&path).unwrap();

        assert_eq!(graph.nodes.len(), 8);
        assert_eq!(graph.arcs.len(), 14);

        assert_eq!(
            weight(
                haversine_distance(0., 0., 0., 1.) * 3.0,
                &RoadType::Secondary
            ),
            graph
                .arcs
                .iter()
                .find(|arc| arc.source == 2 && arc.target == 5)
                .unwrap()
                .weight
        );
    }
}
