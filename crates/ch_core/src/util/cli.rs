use std::path::PathBuf;

use clap::Parser;

use crate::{
    contraction_strategy::{CHStrategy, UpdateStrategy},
    node_contraction::{ContractionParams, PriorityParams},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the .pbf file
    pbf_file: String,

    /// If set the graph will not be simplified
    #[arg(long, default_value = "false")]
    raw_graph: bool,

    /// Set the coefficient for the edge difference term
    #[arg(short, long, value_name = "E")]
    ed: Option<i32>,

    /// Set the coefficient for the contracted neighbors term
    #[arg(short, long, value_name = "C")]
    cn: Option<i32>,

    /// Set the coefficient for the search space depth term
    #[arg(short, long, value_name = "S")]
    ss: Option<i32>,
    /// Set the coefficient for the original edges term
    #[arg(short, long, value_name = "O")]
    oe: Option<i32>,

    /// Set the lazy update strategy. Possible values are "jit" and "local"
    #[arg(long, value_name = "strategy")]
    strat: Option<String>,

    /// Enable or disable periodic updates
    #[arg(short, long, value_name = "periodic")]
    periodic: bool,
}

#[derive(Debug, Clone)]
pub struct Cfg<'a> {
    pub pbf_file: PathBuf,
    pub simplify: bool,
    pub params: ContractionParams,
    pub strategy: CHStrategy<'a>,
}

pub fn parse<'a>() -> Cfg<'a> {
    let cli = Cli::parse();

    let pbf_file = cli.pbf_file;

    let mut priority_params = PriorityParams::default();

    if let Some(ed) = cli.ed {
        priority_params = priority_params.edge_difference_coeff(ed);
    }
    if let Some(cn) = cli.cn {
        priority_params = priority_params.contracted_neighbors_coeff(cn);
    }
    if let Some(ss) = cli.ss {
        priority_params = priority_params.search_space_coeff(ss);
    }
    if let Some(oe) = cli.oe {
        priority_params = priority_params.original_edges_coeff(oe);
    }

    let mut lazy_strategy = UpdateStrategy::default();

    match cli.strat.as_deref() {
        Some("jit") => {
            lazy_strategy = lazy_strategy.set_update_neighbors(false);
        }
        Some("local") => {
            lazy_strategy = lazy_strategy
                .set_update_top(false)
                .set_update_neighbors(true);
        }
        _ => {}
    };

    if cli.periodic {
        lazy_strategy = lazy_strategy.set_periodic_updates(true);
    }

    Cfg {
        pbf_file: PathBuf::from(pbf_file),
        params: ContractionParams::new().priority_params(priority_params),
        strategy: CHStrategy::LazyUpdate(lazy_strategy),
        simplify: !cli.raw_graph,
    }
}
