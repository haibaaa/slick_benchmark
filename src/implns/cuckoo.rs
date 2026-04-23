//! Two-table cuckoo hashing baseline.

use crate::hash_utils::{hash1, hash2};
use crate::trait_def::HashTable;
use std::hash::Hash;

const MAX_KICKS: usize = 128;

/// Cuckoo table with one location in each backing array.
pub struct CuckooTable<K> {
    table1: Vec<Option<K>>,
    table2: Vec<Option<K>>,
    capacity: usize,
    count: usize,
}

impl<K: Hash + Eq + Clone> CuckooTable<K> {
    fn idx1(key: &K, cap: usize) -> usize {
        (hash1(key) as usize) % cap
    }

    fn idx2(key: &K, cap: usize) -> usize {
        (hash2(key) as usize) % cap
    }

    /// Attempts one cuckoo insertion sequence before signaling a rebuild.
    fn try_insert(
        t1: &mut [Option<K>],
        t2: &mut [Option<K>],
        cap: usize,
        mut key: K,
    ) -> Result<(), K> {
        for _ in 0..MAX_KICKS {
            let i1 = Self::idx1(&key, cap);
            if t1[i1].is_none() {
                t1[i1] = Some(key);
                return Ok(());
            }
            key = t1[i1].replace(key).unwrap();

            let i2 = Self::idx2(&key, cap);
            if t2[i2].is_none() {
                t2[i2] = Some(key);
                return Ok(());
            }
            key = t2[i2].replace(key).unwrap();
        }
        Err(key)
    }

    /// Grows the tables until all existing keys can be reinserted successfully.
    fn rebuild(&mut self) {
        let mut new_cap = self.capacity * 2;
        loop {
            let mut new_t1 = vec![None; new_cap];
            let mut new_t2 = vec![None; new_cap];
            let old1 = std::mem::take(&mut self.table1);
            let old2 = std::mem::take(&mut self.table2);
            let mut failed = false;

            for k in old1.into_iter().chain(old2.into_iter()).flatten() {
                if Self::try_insert(&mut new_t1, &mut new_t2, new_cap, k).is_err() {
                    failed = true;
                    break;
                }
            }

            if !failed {
                self.table1 = new_t1;
                self.table2 = new_t2;
                self.capacity = new_cap;
                return;
            }

            new_cap *= 2;
        }
    }
}

impl<K: Hash + Eq + Clone> HashTable<K> for CuckooTable<K> {
    fn new(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two().max(16);
        CuckooTable {
            table1: vec![None; capacity],
            table2: vec![None; capacity],
            capacity,
            count: 0,
        }
    }

    fn insert(&mut self, key: K) {
        if self.find(&key) {
            return;
        }
        loop {
            match Self::try_insert(
                &mut self.table1,
                &mut self.table2,
                self.capacity,
                key.clone(),
            ) {
                Ok(()) => {
                    self.count += 1;
                    return;
                }
                Err(_) => self.rebuild(),
            }
        }
    }

    fn find(&self, key: &K) -> bool {
        let i1 = Self::idx1(key, self.capacity);
        let i2 = Self::idx2(key, self.capacity);
        self.table1[i1].as_ref().is_some_and(|k| k == key)
            || self.table2[i2].as_ref().is_some_and(|k| k == key)
    }

    fn capacity(&self) -> usize {
        self.capacity * 2
    }

    fn len(&self) -> usize {
        self.count
    }

    fn extra_space(&self) -> usize {
        0
    }
}
