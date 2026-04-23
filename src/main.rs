use clap::Parser;

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

fn run_bulk_u64(
    dataset: &slickbench::datasets::Dataset<u64>,
    reps: usize,
) -> Vec<slickbench::metrics::record::BenchRecord> {
    let config = slickbench::runner::bench::RunConfig {
        initial_capacity: 1024,
        repetitions: reps,
    };
    vec![
        slickbench::runner::bench::run_one::<u64, slickbench::implns::linear::LinearTable<u64>, _>(
            &config,
            dataset,
            "bulk",
            "linear",
            slickbench::workloads::bulk::run,
        ),
        slickbench::runner::bench::run_one::<
            u64,
            slickbench::implns::quadratic::QuadraticTable<u64>,
            _,
        >(
            &config,
            dataset,
            "bulk",
            "quadratic",
            slickbench::workloads::bulk::run,
        ),
        slickbench::runner::bench::run_one::<u64, slickbench::implns::cuckoo::CuckooTable<u64>, _>(
            &config,
            dataset,
            "bulk",
            "cuckoo",
            slickbench::workloads::bulk::run,
        ),
        slickbench::runner::bench::run_one::<u64, slickbench::implns::slick::SlickHash<u64>, _>(
            &config,
            dataset,
            "bulk",
            "slick",
            slickbench::workloads::bulk::run,
        ),
        slickbench::runner::bench::run_one::<u64, slickbench::implns::std_set::StdSetTable<u64>, _>(
            &config,
            dataset,
            "bulk",
            "std_set",
            slickbench::workloads::bulk::run,
        ),
    ]
}

fn run_bulk_string(
    dataset: &slickbench::datasets::Dataset<String>,
    reps: usize,
) -> Vec<slickbench::metrics::record::BenchRecord> {
    let config = slickbench::runner::bench::RunConfig {
        initial_capacity: 1024,
        repetitions: reps,
    };
    vec![
        slickbench::runner::bench::run_one::<
            String,
            slickbench::implns::linear::LinearTable<String>,
            _,
        >(
            &config,
            dataset,
            "bulk",
            "linear",
            slickbench::workloads::bulk::run,
        ),
        slickbench::runner::bench::run_one::<
            String,
            slickbench::implns::quadratic::QuadraticTable<String>,
            _,
        >(
            &config,
            dataset,
            "bulk",
            "quadratic",
            slickbench::workloads::bulk::run,
        ),
        slickbench::runner::bench::run_one::<
            String,
            slickbench::implns::cuckoo::CuckooTable<String>,
            _,
        >(
            &config,
            dataset,
            "bulk",
            "cuckoo",
            slickbench::workloads::bulk::run,
        ),
        slickbench::runner::bench::run_one::<String, slickbench::implns::slick::SlickHash<String>, _>(
            &config,
            dataset,
            "bulk",
            "slick",
            slickbench::workloads::bulk::run,
        ),
        slickbench::runner::bench::run_one::<
            String,
            slickbench::implns::std_set::StdSetTable<String>,
            _,
        >(
            &config,
            dataset,
            "bulk",
            "std_set",
            slickbench::workloads::bulk::run,
        ),
    ]
}

fn main() {
    let cli = Cli::parse();
    if cli.workload != "bulk" {
        panic!(
            "unsupported workload '{}': Phase 3 only supports bulk",
            cli.workload
        );
    }

    let records = match cli.dataset.as_str() {
        "uniform" => run_bulk_u64(
            &slickbench::datasets::uniform::generate(cli.size, cli.seed),
            cli.reps,
        ),
        "sequential" => run_bulk_u64(
            &slickbench::datasets::sequential::generate(cli.size, cli.seed),
            cli.reps,
        ),
        "zipf" => run_bulk_u64(
            &slickbench::datasets::zipf::generate(cli.size, cli.seed),
            cli.reps,
        ),
        "norvig" => run_bulk_string(
            &slickbench::datasets::norvig::load(cli.size, cli.seed),
            cli.reps,
        ),
        "wikipedia" => run_bulk_string(
            &slickbench::datasets::wikipedia::load(cli.size, cli.seed),
            cli.reps,
        ),
        other => panic!("unsupported dataset '{other}'"),
    };

    slickbench::metrics::record::write_csv(&cli.output, &records).unwrap();
    println!(
        "Phase 3 complete. dataset={}, workload={}, size={}",
        cli.dataset, cli.workload, cli.size
    );
}
