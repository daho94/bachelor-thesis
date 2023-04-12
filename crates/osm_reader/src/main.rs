use std::path::Path;

use osm_reader::*;

fn main() -> anyhow::Result<()> {
    let now = std::time::Instant::now();
    // Read path to file from command line
    let pbf_path = std::env::args().nth(1).expect("No path to PBF file given");

    // Read PBF file
    let graph = RoadGraph::from_pbf(Path::new(&pbf_path))?;
    graph.write_csv()?;

    let elapsed = now.elapsed();

    println!(
        "Finished reading PBF file in {}.{:03} seconds",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );
    println!(
        "Graph has {} nodes and {} edges",
        graph.get_nodes().len(),
        graph.get_arcs().len()
    );
    Ok(())
}
