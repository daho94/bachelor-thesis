show-vaterstetten:
  RUST_LOG=info cargo r --release -p graph_viz -- crates/osm_reader/test_data/vaterstetten_pp.osm.pbf  

show-saarland:
  RUST_LOG=info cargo r --release -p graph_viz -- crates/osm_reader/test_data/saarland_pp.osm.pbf

show PBF_FILE:
  RUST_LOG=info cargo r --release -p graph_viz -- {{PBF_FILE}}
