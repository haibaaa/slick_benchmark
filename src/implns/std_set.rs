//! `std::collections::HashSet` baseline with a fixed ahash seed.

use crate::trait_def::HashTable;
use std::hash::Hash;

/// Standard-library hash set used as a practical baseline.
pub struct StdSetTable<K> {
    inner: std::collections::HashSet<K, ahash::RandomState>,
}

impl<K: Hash + Eq + Clone> HashTable<K> for StdSetTable<K> {
    fn new(capacity: usize) -> Self {
        let state = ahash::RandomState::with_seeds(
            crate::hash_utils::SEED1.0,
            crate::hash_utils::SEED1.1,
            crate::hash_utils::SEED2.0,
            crate::hash_utils::SEED2.1,
        );
        StdSetTable {
            inner: std::collections::HashSet::with_capacity_and_hasher(capacity, state),
        }
    }

    fn insert(&mut self, key: K) {
        self.inner.insert(key);
    }

    fn find(&self, key: &K) -> bool {
        self.inner.contains(key)
    }

    fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn extra_space(&self) -> usize {
        0
    }
}
