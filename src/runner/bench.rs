use crate::datasets::Dataset;
use crate::metrics::record::BenchRecord;
use crate::trait_def::HashTable;
use crate::workloads::WorkloadResult;
use std::hash::Hash;

/// Configuration for a single benchmark run.
pub struct RunConfig {
    /// Initial capacity hint passed to HashTable::new()
    pub initial_capacity: usize,
    /// Number of times to repeat each workload (take minimum for stability)
    pub repetitions: usize,
}

impl Default for RunConfig {
    fn default() -> Self {
        RunConfig {
            initial_capacity: 1024,
            repetitions: 3,
        }
    }
}

/// Run a workload function on a table and return the BenchRecord.
pub fn run_one<K, T, F>(
    config: &RunConfig,
    dataset: &Dataset<K>,
    workload_name: &str,
    table_name: &str,
    workload_fn: F,
) -> BenchRecord
where
    K: Hash + Eq + Clone,
    T: HashTable<K>,
    F: Fn(&mut T, &Dataset<K>) -> WorkloadResult,
{
    let mut best: Option<WorkloadResult> = None;

    for _ in 0..config.repetitions {
        let mut table = T::new(config.initial_capacity);
        let result = workload_fn(&mut table, dataset);
        best = Some(match best {
            None => result,
            Some(prev) => WorkloadResult {
                insert_ns: prev.insert_ns.min(result.insert_ns),
                find_ns: prev.find_ns.min(result.find_ns),
                insert_count: result.insert_count,
                find_count: result.find_count,
            },
        });
    }

    let best = best.unwrap();
    let load_factor = dataset.keys.len() as f64 / config.initial_capacity as f64;

    BenchRecord::from_result(&dataset.name, workload_name, table_name, load_factor, &best)
}
