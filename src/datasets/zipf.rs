use crate::datasets::Dataset;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_distr::{Distribution, Zipf};

/// Generates `size` u64 keys from a Zipf distribution.
/// exponent = 1.1 (configurable, but default to 1.1 for skewed real-world approximation).
/// Keys map directly from Zipf samples (1-indexed u64).
pub fn generate(size: usize, seed: u64) -> Dataset<u64> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let zipf = Zipf::new(size as u64, 1.1).expect("Zipf distribution creation failed");
    let mut keys: Vec<u64> = (0..size).map(|_| zipf.sample(&mut rng) as u64).collect();
    keys.shuffle(&mut rng);
    Dataset {
        name: "zipf".to_string(),
        keys,
    }
}
