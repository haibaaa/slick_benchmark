//! Read-heavy workload with 95% finds and 5% inserts.

use crate::datasets::Dataset;
use crate::trait_def::HashTable;
use crate::workloads::WorkloadResult;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::hash::Hash;
use std::time::Instant;

const SEED: u64 = 0xDEAD_BEEF_CAFE_1234;
const FIND_RATIO: f64 = 0.95;

/// Read-heavy workload: 95% find, 5% insert.
pub fn run<K, T>(table: &mut T, dataset: &Dataset<K>) -> WorkloadResult
where
    K: Hash + Eq + Clone,
    T: HashTable<K>,
{
    let keys = &dataset.keys;
    let n = keys.len();
    let half = n / 2;

    // Warm-up is intentionally excluded from timing so the measured portion
    // reflects the read-dominant steady state.
    for key in &keys[..half] {
        table.insert(key.clone());
    }

    let mut rng = SmallRng::seed_from_u64(SEED);
    let mut insert_ns = 0u64;
    let mut find_ns = 0u64;
    let mut insert_count = 0usize;
    let mut find_count = 0usize;
    let mut insert_idx = half;
    let mut find_idx = 0usize;

    // Per-operation timing is required here because inserts and finds are
    // interleaved rather than executed in contiguous phases.
    for _ in 0..half {
        if rng.gen::<f64>() > FIND_RATIO && insert_idx < n {
            let t = Instant::now();
            table.insert(keys[insert_idx].clone());
            insert_ns += t.elapsed().as_nanos() as u64;
            insert_count += 1;
            insert_idx += 1;
        } else {
            let key = &keys[find_idx % insert_idx.max(1)];
            let t = Instant::now();
            let _ = table.find(key);
            find_ns += t.elapsed().as_nanos() as u64;
            find_count += 1;
            find_idx += 1;
        }
    }

    WorkloadResult {
        insert_ns,
        find_ns,
        insert_count,
        find_count,
    }
}
