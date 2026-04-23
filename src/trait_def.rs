/// Common interface for all hash table implementations.
///
/// RULES:
/// - Do NOT add Slick-specific methods here.
/// - Do NOT add statistics/diagnostics methods here.
/// - Keep this trait minimal: new, insert, find.
/// - Hashing is always internal to the implementation (not passed in).
///
/// K must be Hash + Eq + Clone.
/// Clone is required so datasets can be reused across multiple table instances.
pub trait HashTable<K: std::hash::Hash + Eq + Clone> {
    /// Create a new table with the given initial capacity.
    /// Capacity is a hint; the table may resize.
    fn new(capacity: usize) -> Self;

    /// Insert a key into the table.
    /// If the key already exists, behavior is implementation-defined
    /// (idempotent insert is acceptable; duplicate tracking is not required).
    fn insert(&mut self, key: K);

    /// Return true if key exists in the table, false otherwise.
    fn find(&self, key: &K) -> bool;

    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
    fn extra_space(&self) -> usize;
}
