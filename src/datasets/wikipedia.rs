//! Loader for the Wikipedia title dataset.

use crate::datasets::Dataset;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

/// Loads titles from data/wiki_titles.txt (one title per line).
/// Returns at most `size` titles.
pub fn load(size: usize, seed: u64) -> Dataset<String> {
    let content = std::fs::read_to_string("data/wiki_titles.txt")
        .expect("data/wiki_titles.txt not found. Run: uv run scripts/download_data.py");
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut keys: Vec<String> = content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .take(size)
        .collect();
    keys.shuffle(&mut rng);
    Dataset {
        name: "wikipedia".to_string(),
        keys,
    }
}
