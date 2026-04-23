/// A loaded dataset, ready for benchmarking.
/// Keys are already shuffled and NOT deduplicated.
pub struct Dataset<K> {
    pub name: String,
    pub keys: Vec<K>,
}

pub mod norvig;
pub mod sequential;
pub mod uniform;
pub mod wikipedia;
pub mod zipf;
