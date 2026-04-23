use crate::datasets::Dataset;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

/// Generates `size` uniformly random u64 values.
/// Keys are NOT deduplicated — duplicates are expected and preserved.
pub fn generate(size: usize, seed: u64) -> Dataset<u64> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut keys: Vec<u64> = (0..size).map(|_| rng.gen::<u64>()).collect();
    keys.shuffle(&mut rng);
    Dataset {
        name: "uniform".to_string(),
        keys,
    }
}
