use std::{fs::File, path::PathBuf};

use osm_reader::RoadGraph;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let args = std::env::args().skip(1).filter(|p| p.ends_with(".pbf"));

    let mut file = File::create("simplify.csv")?;
    writeln!(&mut file, "file,nodes,edges,nodes_simple,edges_simple")?;

    for pbf_path in args {
        let path_buf = PathBuf::from(&pbf_path);
        let ((nodes, edges), (nodes_simple, edges_simple)) = run_bench(&pbf_path);
        writeln!(
            &mut file,
            "{},{},{},{},{}",
            path_buf.iter().last().unwrap().to_str().unwrap(),
            nodes,
            edges,
            nodes_simple,
            edges_simple
        )?;
    }

    Ok(())
}

fn run_bench<P: Into<PathBuf>>(pbf_path: P) -> ((usize, usize), (usize, usize)) {
    let path = &pbf_path.into();
    dbg!(&path);
    let g = RoadGraph::from_pbf(path).unwrap();

    let g_simple = RoadGraph::from_pbf_with_simplification(path).unwrap();

    println!(
        "Graph: #nodes {}, #edges {}",
        g.get_nodes().len(),
        g.get_arcs().len()
    );
    println!(
        "Simplyfied Graph: #nodes {}, #edges {}",
        g_simple.get_nodes().len(),
        g_simple.get_arcs().len()
    );
    (
        (g.get_nodes().len(), g.get_arcs().len()),
        (g_simple.get_nodes().len(), g_simple.get_arcs().len()),
    )
}
