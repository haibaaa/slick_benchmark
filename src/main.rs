//! CLI entry point for the benchmark binary.
//!
//! The binary parses a dataset/workload selection, instantiates the matching
//! typed dataset, and forwards execution to the shared generic runner.

use clap::Parser;
use slickbench::datasets::Dataset;
use slickbench::metrics::record::BenchRecord;
use std::hash::Hash;

#[derive(Parser, Debug)]
#[command(name = "slickbench", about = "Hash table benchmarking framework")]
struct Cli {
    /// Dataset to use: uniform | sequential | zipf | norvig | wikipedia
    #[arg(long, default_value = "uniform")]
    dataset: String,

    /// Workload to use: bulk | mixed | read_heavy
    #[arg(long, default_value = "bulk")]
    workload: String,

    /// Number of keys to generate/load
    #[arg(long, default_value_t = 1_000_000)]
    size: usize,

    /// Random seed for dataset generation
    #[arg(long, default_value_t = 42)]
    seed: u64,

    /// Output CSV file path
    #[arg(long, default_value = "results.csv")]
    output: String,

    /// Number of repetitions per (table, workload) pair
    #[arg(long, default_value_t = 3)]
    reps: usize,
}

/// Expands to one benchmark run per table implementation for a specific
/// dataset/workload pairing while preserving monomorphized dispatch.
macro_rules! run_all_tables_for_workload {
    ($config:expr, $dataset:expr, $workload_name:expr, $workload_fn:path) => {
        vec![
            slickbench::runner::bench::run_one::<K, slickbench::implns::linear::LinearTable<K>, _>(
                $config,
                $dataset,
                $workload_name,
                "linear",
                $workload_fn,
            ),
            slickbench::runner::bench::run_one::<
                K,
                slickbench::implns::quadratic::QuadraticTable<K>,
                _,
            >($config, $dataset, $workload_name, "quadratic", $workload_fn),
            slickbench::runner::bench::run_one::<K, slickbench::implns::cuckoo::CuckooTable<K>, _>(
                $config,
                $dataset,
                $workload_name,
                "cuckoo",
                $workload_fn,
            ),
            slickbench::runner::bench::run_one::<K, slickbench::implns::slick::SlickHash<K>, _>(
                $config,
                $dataset,
                $workload_name,
                "slick",
                $workload_fn,
            ),
            slickbench::runner::bench::run_one::<K, slickbench::implns::std_set::StdSetTable<K>, _>(
                $config,
                $dataset,
                $workload_name,
                "std_set",
                $workload_fn,
            ),
        ]
    };
}

/// Executes one workload across all table implementations for a dataset type.
fn run_workload<K>(dataset: &Dataset<K>, reps: usize, workload: &str) -> Vec<BenchRecord>
where
    K: Hash + Eq + Clone + Default,
{
    let config = slickbench::runner::bench::RunConfig {
        initial_capacity: dataset.keys.len(),
        repetitions: reps,
    };

    match workload {
        "bulk" => {
            run_all_tables_for_workload!(&config, dataset, "bulk", slickbench::workloads::bulk::run)
        }
        "mixed" => run_all_tables_for_workload!(
            &config,
            dataset,
            "mixed",
            slickbench::workloads::mixed::run
        ),
        "read_heavy" => run_all_tables_for_workload!(
            &config,
            dataset,
            "read_heavy",
            slickbench::workloads::read_heavy::run
        ),
        other => panic!("unsupported workload '{other}'"),
    }
}

fn main() {
    let cli = Cli::parse();
    // Dataset dispatch remains typed so both numeric and string-key tables are
    // compiled through the same generic workload path.
    let records = match cli.dataset.as_str() {
        "uniform" => run_workload(
            &slickbench::datasets::uniform::generate(cli.size, cli.seed),
            cli.reps,
            &cli.workload,
        ),
        "sequential" => run_workload(
            &slickbench::datasets::sequential::generate(cli.size, cli.seed),
            cli.reps,
            &cli.workload,
        ),
        "zipf" => run_workload(
            &slickbench::datasets::zipf::generate(cli.size, cli.seed),
            cli.reps,
            &cli.workload,
        ),
        "norvig" => run_workload(
            &slickbench::datasets::norvig::load(cli.size, cli.seed),
            cli.reps,
            &cli.workload,
        ),
        "wikipedia" => run_workload(
            &slickbench::datasets::wikipedia::load(cli.size, cli.seed),
            cli.reps,
            &cli.workload,
        ),
        other => panic!("unsupported dataset '{other}'"),
    };

    slickbench::metrics::record::write_csv(&cli.output, &records).unwrap();
    println!(
        "Phase 5 complete. dataset={}, workload={}, size={}",
        cli.dataset, cli.workload, cli.size
    );
}
