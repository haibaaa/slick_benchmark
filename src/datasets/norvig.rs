//! Loader for the Norvig word-frequency dataset.

use crate::datasets::Dataset;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

/// Loads words from data/norvig_words.txt.
/// Each line is: `<word>\t<count>` — we take only the word.
/// Returns at most `size` words (or all words if the file has fewer).
pub fn load(size: usize, seed: u64) -> Dataset<String> {
    let content = std::fs::read_to_string("data/norvig_words.txt")
        .expect("data/norvig_words.txt not found. Run: uv run scripts/download_data.py");
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut keys: Vec<String> = content
        .lines()
        .filter_map(|line| line.split('\t').next().map(|w| w.to_string()))
        .take(size)
        .collect();
    keys.shuffle(&mut rng);
    Dataset {
        name: "norvig".to_string(),
        keys,
    }
}
