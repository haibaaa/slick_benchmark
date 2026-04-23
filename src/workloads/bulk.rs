//! Bulk workload: insert all keys, then probe all keys.

use crate::datasets::Dataset;
use crate::trait_def::HashTable;
use crate::workloads::WorkloadResult;
use std::hash::Hash;
use std::time::Instant;

/// Phase 1 workload: insert all keys, then find all keys.
pub fn run<K, T>(table: &mut T, dataset: &Dataset<K>) -> WorkloadResult
where
    K: Hash + Eq + Clone,
    T: HashTable<K>,
{
    let keys = &dataset.keys;
    let n = keys.len();

    // Bulk timings are batched by phase to avoid per-operation timer overhead.
    let t0 = Instant::now();
    for key in keys {
        table.insert(key.clone());
    }
    let insert_ns = t0.elapsed().as_nanos() as u64;

    let t1 = Instant::now();
    for key in keys {
        let _ = table.find(key);
    }
    let find_ns = t1.elapsed().as_nanos() as u64;

    WorkloadResult {
        insert_ns,
        find_ns,
        insert_count: n,
        find_count: n,
    }
}
