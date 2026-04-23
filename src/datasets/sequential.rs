//! Sequential `u64` dataset generator.

use crate::datasets::Dataset;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

/// Generates keys [0, 1, 2, ..., size-1] then shuffles them.
/// Represents a cache-friendly, low-entropy distribution.
pub fn generate(size: usize, seed: u64) -> Dataset<u64> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut keys: Vec<u64> = (0u64..size as u64).collect();
    keys.shuffle(&mut rng);
    Dataset {
        name: "sequential".to_string(),
        keys,
    }
}
