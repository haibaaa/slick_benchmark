//! Core benchmark runner utilities.
//!
//! The runner executes one workload across repeated fresh table instances and
//! retains the best observed insert and find times independently.

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
    let mut best_metrics: Option<(usize, usize, usize)> = None;

    for _ in 0..config.repetitions {
        let mut table = T::new(config.initial_capacity);
        let result = workload_fn(&mut table, dataset);
        let cap = table.capacity();
        let len = table.len();
        let extra = table.extra_space();
        
        best = Some(match best {
            None => {
                best_metrics = Some((cap, len, extra));
                result
            },
            // The benchmark records the minimum insert and find times seen across
            // repetitions to reduce noise from transient host activity.
            Some(prev) => WorkloadResult {
                insert_ns: prev.insert_ns.min(result.insert_ns),
                find_ns: prev.find_ns.min(result.find_ns),
                insert_count: result.insert_count,
                find_count: result.find_count,
            },
        });
    }

    let best = best.unwrap();
    let metrics = best_metrics.unwrap();
    // Use actual table capacity for load factor, not initial hint
    let load_factor = if metrics.0 > 0 {
        metrics.1 as f64 / metrics.0 as f64
    } else {
        0.0
    };
    
    let bytes_estimate = metrics.0 * std::mem::size_of::<(K, ())>() + metrics.2 * std::mem::size_of::<K>();
    let bytes_per_element = if metrics.1 > 0 { bytes_estimate / metrics.1 } else { 0 };

    BenchRecord::from_result(&dataset.name, workload_name, table_name, load_factor, &best, metrics.0, metrics.1, bytes_estimate, bytes_per_element)
}
